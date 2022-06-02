use std::{collections::{HashMap, VecDeque}, rc::Rc};

use crate::{MipsyResult, Safe, Uninitialised, TEXT_BOT, compile::TEXT_TOP, GLOBAL_BOT, HEAP_BOT, STACK_BOT, STACK_TOP, KTEXT_BOT, MipsyError, error::runtime::{RuntimeError, SegmentationFaultAccessType, self}};
use super::{PAGE_SIZE, SafeToUninitResult};

pub const WRITE_MARKER_LO: u32 = 32;
pub const WRITE_MARKER_HI: u32 = 32;

pub const TIMELINE_MAX_LEN: usize = 1_000_000;

/// # A timeline of states
#[derive(Default)]
pub struct Timeline {
    seed: State,
    timeline: VecDeque<State>,
    lost_history: bool,
}

impl Timeline {
    pub fn new(seed: State) -> Self {
        let timeline = VecDeque::new();

        Self {
            seed,
            timeline,
            lost_history: false,
        }
    }

    pub fn state(&self) -> &State {
        self.timeline.back().unwrap_or(&self.seed)
    }

    pub fn state_mut(&mut self) -> &mut State {
        self.timeline.back_mut().unwrap_or(&mut self.seed)
    }

    pub fn reset(&mut self) {
        self.lost_history = false;
        self.timeline = VecDeque::new();
    }

    pub fn timeline_len(&self) -> usize {
        self.timeline.len() + 1
    }

    pub fn nth_state(&self, n: usize) -> Option<&State> {
        if n == 0 {
            Some(&self.seed)
        } else {
            self.timeline.get(n - 1)
        }
    }

    pub fn prev_state(&self) -> Option<&State> {
        let len = self.timeline_len();
        if len == 1 {
            None
        } else {
            self.nth_state(len - 2)
        }
    }

    pub fn push_next_state(&mut self) -> &mut State {
        let timeline_len = self.timeline_len();
        if timeline_len == self.timeline.capacity()
            && timeline_len     < TIMELINE_MAX_LEN
            && timeline_len * 2 > TIMELINE_MAX_LEN {
            self.timeline.reserve_exact(TIMELINE_MAX_LEN - timeline_len);
        }

        if self.timeline_len() >= TIMELINE_MAX_LEN {
            self.timeline.pop_front();
            self.lost_history = true;
        }

        let last_state = self.timeline.back().unwrap_or(&self.seed);
        let next_state = last_state.clone();

        self.timeline.push_back(next_state);

        self.timeline.back_mut().expect("just pushed to the timeline")
    }

    pub fn pop_last_state(&mut self) -> bool {
        if !self.timeline.is_empty() {
            self.timeline.pop_back();
            true
        } else {
            false
        }
    }

    pub fn lost_history(&self) -> bool {
        self.lost_history
    }
}

pub struct State {
    pub(super) pages: HashMap<u32, Rc<[Safe<u8>; PAGE_SIZE]>>,
    pub(super) pc: u32,
    pub(super) registers: [Safe<i32>; 32],
    pub(super) write_marker: u64,
    pub(super) hi: Safe<i32>,
    pub(super) lo: Safe<i32>,
    pub(super) heap_size: u32,
}

impl State {
    pub fn pc(&self) -> u32 {
        self.pc
    }

    pub fn set_pc(&mut self, pc: u32) {
        self.pc = pc;
    }

    pub fn heap_size(&self) -> u32 {
        self.heap_size
    }

    pub fn set_heap_size(&mut self, heap_size: u32) {
        self.heap_size = heap_size;
    }

    pub fn write_marker(&self) -> u64 {
        self.write_marker
    }

    pub fn set_write_marker(&mut self, write_marker: u64) {
        self.write_marker = write_marker;
    }

    pub fn registers(&self) -> &[Safe<i32>] {
       &self.registers
    }

    pub fn read_register(&self, reg_num: u32) -> MipsyResult<i32> {
        self.registers[reg_num as usize]
            .to_result(Uninitialised::Register { reg_num })
    }

    pub fn read_register_uninit(&self, reg_num: u32) -> Safe<i32> {
        self.registers[reg_num as usize]
    }

    pub fn read_hi(&self) -> MipsyResult<i32> {
        self.hi
            .to_result(Uninitialised::Hi)
    }

    pub fn read_lo(&self) -> MipsyResult<i32> {
        self.lo
            .to_result(Uninitialised::Lo)
    }

    pub fn write_register(&mut self, reg_num: u32, value: i32) {
        if reg_num == 0 {
            return;
        }

        assert!(reg_num < 32);

        self.registers[reg_num as usize] = Safe::Valid(value);
        self.write_marker |= 1u64 << reg_num;
    }

    pub fn write_register_uninit(&mut self, reg_num: u32, value: Safe<i32>) {
        if reg_num == 0 {
            return;
        }

        assert!(reg_num < 32);

        self.registers[reg_num as usize] = value;
        self.write_marker |= 1u64 << reg_num;
    }

    pub fn write_hi(&mut self, value: i32) {
        self.hi = Safe::Valid(value);
        self.write_marker |= 1u64 << WRITE_MARKER_HI;
    }

    pub fn write_lo(&mut self, value: i32) {
        self.lo = Safe::Valid(value);
        self.write_marker |= 1u64 << WRITE_MARKER_LO;
    }

    pub fn check_segfault(&self, address: u32, access: SegmentationFaultAccessType) -> MipsyResult<()> {
        let segfault = match address {
            // TODO(zkol): Update this when exclusive range matching is stabilised
            _ if address < TEXT_BOT => {
                true
            }
            _ if (TEXT_BOT..=TEXT_TOP).contains(&address) => {
                false
            }
            _ if (GLOBAL_BOT..HEAP_BOT).contains(&address) => {
                false
            }
            _ if (HEAP_BOT..STACK_BOT).contains(&address) => {
                let heap_offset = address - HEAP_BOT;

                heap_offset >= self.heap_size()
            }
            _ if (STACK_BOT..=STACK_TOP).contains(&address) => {
                false
            }
            _ if address >= KTEXT_BOT => {
                self.pc() < KTEXT_BOT
            }
            _ => unreachable!(),
        };

        if segfault {
            Err(MipsyError::Runtime(RuntimeError::new(runtime::Error::SegmentationFault { addr: address, access })))
        } else {
            Ok(())
        }
    }

    pub fn read_mem_byte(&self, address: u32) -> MipsyResult<u8> {
        self.check_segfault(address, SegmentationFaultAccessType::Read)?;

        self.get_page(address)
            .and_then(|page| {
                let offset = Self::offset_in_page(address);

                page[offset as usize].as_option().copied()
            })
            .to_result(Uninitialised::Byte { addr: address })
    }

    pub fn read_mem_half(&self, address: u32) -> MipsyResult<u16> {
        let result: MipsyResult<_> = (|| {
            let byte1 = self.read_mem_byte(address)?;
            let byte2 = self.read_mem_byte(address + 1)?;

            Ok(u16::from_le_bytes([byte1, byte2]))
        })();

        result.ok().to_result(Uninitialised::Half { addr: address })
    }

    pub fn read_mem_word(&self, address: u32) -> MipsyResult<u32> {
        let result: MipsyResult<_> = (|| {
            let byte1 = self.read_mem_byte(address)?;
            let byte2 = self.read_mem_byte(address + 1)?;
            let byte3 = self.read_mem_byte(address + 2)?;
            let byte4 = self.read_mem_byte(address + 3)?;

            Ok(u32::from_le_bytes([byte1, byte2, byte3, byte4]))
        })();

        result.ok().to_result(Uninitialised::Word { addr: address })
    }

    pub fn read_mem_byte_uninit(&self, address: u32) -> MipsyResult<Safe<u8>> {
        self.check_segfault(address, SegmentationFaultAccessType::Read)?;

        Ok(
            self.get_page(address)
                .and_then(|page| {
                    let offset = Self::offset_in_page(address);

                    page[offset as usize].as_option().copied()
                })
                .map(Safe::Valid)
                .unwrap_or(Safe::Uninitialised)
        )
    }

    pub fn read_mem_half_uninit(&self, address: u32) -> MipsyResult<Safe<u16>> {
        self.check_segfault(address, SegmentationFaultAccessType::Read)?;
        self.check_segfault(address + 2, SegmentationFaultAccessType::Read)?;

        let result: MipsyResult<_> = (|| {
            let byte1 = self.read_mem_byte(address)?;
            let byte2 = self.read_mem_byte(address + 1)?;

            Ok(u16::from_le_bytes([byte1, byte2]))
        })();

        Ok(
            result.map(Safe::Valid)
                .unwrap_or(Safe::Uninitialised)
        )
    }

    pub fn read_mem_word_uninit(&self, address: u32) -> MipsyResult<Safe<u32>> {
        self.check_segfault(address, SegmentationFaultAccessType::Read)?;
        self.check_segfault(address + 1, SegmentationFaultAccessType::Read)?;
        self.check_segfault(address + 2, SegmentationFaultAccessType::Read)?;
        self.check_segfault(address + 3, SegmentationFaultAccessType::Read)?;

        let result: MipsyResult<_> = (|| {
            let byte1 = self.read_mem_byte(address)?;
            let byte2 = self.read_mem_byte(address + 1)?;
            let byte3 = self.read_mem_byte(address + 2)?;
            let byte4 = self.read_mem_byte(address + 3)?;

            Ok(u32::from_le_bytes([byte1, byte2, byte3, byte4]))
        })();

        Ok(
            result.map(Safe::Valid)
                .unwrap_or(Safe::Uninitialised)
        )
    }

    pub fn write_mem_byte(&mut self, address: u32, byte: u8) -> MipsyResult<()> {
        self.check_segfault(address, SegmentationFaultAccessType::Write)?;

        let page = self.get_mut_page_or_new(address);
        let offset = Self::offset_in_page(address);

        page[offset as usize] = Safe::Valid(byte);

        Ok(())
    }

    pub fn write_mem_half(&mut self, address: u32, half: u16) -> MipsyResult<()> {
        let [b1, b2] = half.to_le_bytes();

        self.write_mem_byte(address, b1)?;
        self.write_mem_byte(address + 1, b2)?;

        Ok(())
    }

    pub fn write_mem_word(&mut self, address: u32, word: u32) -> MipsyResult<()> {
        let [b1, b2, b3, b4] = word.to_le_bytes();

        self.write_mem_byte(address, b1)?;
        self.write_mem_byte(address + 1, b2)?;
        self.write_mem_byte(address + 2, b3)?;
        self.write_mem_byte(address + 3, b4)?;

        Ok(())
    }

    pub fn write_mem_byte_uninit(&mut self, address: u32, byte: Safe<u8>) -> MipsyResult<()> {
        self.check_segfault(address, SegmentationFaultAccessType::Write)?;

        let page = self.get_mut_page_or_new(address);
        let offset = Self::offset_in_page(address);

        page[offset as usize] = byte;

        Ok(())
    }

    pub fn write_mem_half_uninit(&mut self, address: u32, half: Safe<u16>) -> MipsyResult<()> {
        match half {
            Safe::Valid(half) => self.write_mem_half(address, half)?,
            Safe::Uninitialised => {
                self.write_mem_byte_uninit(address,     Safe::Uninitialised)?;
                self.write_mem_byte_uninit(address + 1, Safe::Uninitialised)?;
            }
        }

        Ok(())
    }

    pub fn write_mem_word_uninit(&mut self, address: u32, word: Safe<u32>) -> MipsyResult<()> {
        match word {
            Safe::Valid(word) => self.write_mem_word(address, word)?,
            Safe::Uninitialised => {
                self.write_mem_byte_uninit(address,     Safe::Uninitialised)?;
                self.write_mem_byte_uninit(address + 1, Safe::Uninitialised)?;
                self.write_mem_byte_uninit(address + 2, Safe::Uninitialised)?;
                self.write_mem_byte_uninit(address + 3, Safe::Uninitialised)?;
            }
        }

        Ok(())
    }

    pub fn read_mem_string(&self, address: u32) -> MipsyResult<Vec<u8>> {
        let mut text = vec![];

        let mut pointer = address;
        loop {
            let value = self.read_mem_byte(pointer)?;

            if value == 0 {
                break Ok(text);
            }

            text.push(value);
            pointer += 1;
        }
    }

    pub fn read_mem_bytes(&self, address: u32, len: u32) -> MipsyResult<Vec<u8>> {
        let mut text = vec![];

        for i in 0..len {
            let value = self.read_mem_byte(address + i)?;

            text.push(value);
        }

        Ok(text)
    }

    pub fn branch(&mut self, imm: i16) {
        let imm = imm as i32 - 1; // branch offset is 1-based
        let imm = imm * 4;        // branch offset is in instructions

        let pc_offset = imm as u32;
        self.pc = self.pc.wrapping_add(pc_offset);
    }

    fn get_page_index(address: u32) -> u32 {
        address / (PAGE_SIZE as u32)
    }

    fn offset_in_page(address: u32) -> u32 {
        address % (PAGE_SIZE as u32)
    }

    fn page_base_addr(page: u32) -> u32 {
        page * (PAGE_SIZE as u32)
    }

    fn addr_to_page_base_addr(address: u32) -> u32 {
        Self::page_base_addr(Self::get_page_index(address))
    }

    pub fn pages(&self) -> &HashMap<u32, Rc<[Safe<u8>; PAGE_SIZE]>> {
        &self.pages
    }

    pub fn get_page(&self, address: u32) -> Option<&[Safe<u8>; PAGE_SIZE]> {
        let base_addr = Self::addr_to_page_base_addr(address);

        self.pages.get(&base_addr).map(|page| &**page)
    }

    pub fn get_mut_page_or_new(&mut self, address: u32) -> &mut [Safe<u8>; PAGE_SIZE] {
        let base_addr = Self::addr_to_page_base_addr(address);

        let page = self.pages.entry(base_addr)
            .or_insert_with(|| Rc::new([Default::default(); PAGE_SIZE]));

        Rc::make_mut(page)
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        let cow_pages = self.pages.iter()
                .map(|(&addr, val)| (addr, val.clone()))
                .collect::<HashMap<_, _>>();

        Self {
            pages: cow_pages,
            pc: self.pc,
            registers: self.registers,
            write_marker: 0,
            hi: self.hi,
            lo: self.lo,
            heap_size: self.heap_size,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            pages: HashMap::new(),
            pc: KTEXT_BOT,
            heap_size: 0,
            registers: Default::default(),
            write_marker: 0,
            hi: Default::default(),
            lo: Default::default(),
        }
    }
}
