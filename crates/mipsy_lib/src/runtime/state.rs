use std::{collections::{HashMap, VecDeque}, ops::DerefMut};

use crate::{MipsyResult, Safe, Uninitialised};
use super::{PAGE_SIZE, SafeToUninitResult, unsafe_cow::UnsafeCow};

/// A timeline of states
///
/// # Safety
///
/// Timeline maintains an invariant that for any state (state `a`) in the timeline,
/// and any other state (state `b`) subsequently appended to the timeline,
/// state `a` will live for at least as long as state `b`.
///
/// This follows the standard lifetime subtyping rules in Rust, i.e. `'a: 'b`.
pub struct Timeline {
    timeline: VecDeque<State>,
}

impl Drop for Timeline {
    fn drop(&mut self) {
        // Drop all states in the timeline *in reverse order*.
        // This is important for safety,
        // so that the state invariant is maintained.
        while let Some(state) = self.timeline.pop_back() {}
    }
}

impl Timeline {
    pub fn new(seed: State) -> Self {
        let timeline = VecDeque::with_capacity(1);
        timeline.push_back(seed);

        Self {
            timeline,
        }
    }

    pub fn state(&self) -> &State {
        self.timeline.back().expect("timeline cannot be empty")
    }

    pub fn state_mut(&mut self) -> &mut State {
        self.timeline.back_mut().expect("timeline cannot be empty")
    }

    pub fn reset(&mut self) {
        while self.timeline.len() > 1 {
            self.timeline.pop_back();
        }
    }

    pub fn timeline_len(&self) -> usize {
        self.timeline.len()
    }

    pub fn nth_state(&self, n: usize) -> Option<&State> {
        self.timeline.get(n)
    }

    pub fn push_next_state(&mut self) -> &mut State {
        let last_state = self.timeline.back().expect("timelint cannot be empty");
        self.timeline.push_back(last_state.clone());

        self.timeline.back_mut().expect("just pushed to the timeline")
    }

    pub fn pop_last_state(&mut self) -> bool {
        if self.timeline.len() > 1 {
            self.timeline.pop_back();
            
            true
        } else {
            false
        }
    }
}

pub struct State {
    pages: HashMap<u32, UnsafeCow<[Safe<u8>]>>,
    pc: u32,
    registers: [Safe<i32>; 32],
    hi: Safe<i32>,
    lo: Safe<i32>,
    heap_size: u32,
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

    pub fn read_register(&self, reg_num: u32) -> MipsyResult<i32> {
        self.registers[reg_num as usize]
            .to_result(Uninitialised::Register { reg_num })
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

        self.registers[reg_num as usize] = Safe::Valid(value);
    }

    pub fn write_hi(&mut self, value: i32) {
        self.hi = Safe::Valid(value);
    }

    pub fn write_lo(&mut self, value: i32) {
        self.lo = Safe::Valid(value);
    }

    pub fn read_mem_byte(&self, address: u32) -> MipsyResult<u8> {
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

    pub fn write_mem_byte(&mut self, address: u32, byte: u8) {
        let page = self.get_mut_page_or_new(address);
        let offset = Self::offset_in_page(address);

        page[offset as usize] = Safe::Valid(byte);
    }

    pub fn write_mem_half(&mut self, address: u32, half: u16) {
        let [b1, b2] = half.to_le_bytes();
        
        self.write_mem_byte(address, b1);
        self.write_mem_byte(address + 1, b2);
    }

    pub fn write_mem_word(&mut self, address: u32, word: u32) {
        let [b1, b2, b3, b4] = word.to_le_bytes();
        
        self.write_mem_byte(address, b1);
        self.write_mem_byte(address + 1, b2);
        self.write_mem_byte(address + 2, b3);
        self.write_mem_byte(address + 3, b4);
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
        address / PAGE_SIZE
    }

    fn offset_in_page(address: u32) -> u32 {
        address % PAGE_SIZE
    }

    fn page_base_addr(page: u32) -> u32 {
        page * PAGE_SIZE
    }

    fn addr_to_page_base_addr(address: u32) -> u32 {
        Self::page_base_addr(Self::get_page_index(address))
    }

    pub fn get_page(&self, address: u32) -> Option<&[Safe<u8>]> {
        let base_addr = Self::addr_to_page_base_addr(address);

        self.pages.get(&base_addr).map(|page| {
            // SAFETY: This page will either be owned, or
            //   borrowed from a previous state in the timeline,
            //   which must exist, as Timeline holds an invariant
            //   that each appended state must exist for at least
            //   as long as any further appended states, which is
            //   isomorphic to rust's lifetime subtyping rules.
            unsafe { page.unsafe_borrow() }
        })
    }

    pub fn get_mut_page_or_new(&mut self, address: u32) -> &mut [Safe<u8>] {
        let base_addr = Self::addr_to_page_base_addr(address);

        let entry = self.pages.entry(base_addr)
            .or_insert_with(|| UnsafeCow::new_boxed(Box::new([Default::default(); PAGE_SIZE as usize])))
            .deref_mut();

        // SAFETY: Same argument as Self::get_page,
        //   and mutability is safe because
        //   the reference's lifetime is tied
        //   to our &mut self.
        unsafe { entry.unsafe_borrow_mut_slice() }
    }
}

impl Clone for State {
    fn clone(&self) -> Self {
        let cow_pages = self.pages.into_iter()
                .map(|(addr, val)| (addr, val.to_borrowed()))
                .collect::<HashMap<_, _>>();

        Self {
            pages: cow_pages,
            pc: self.pc,
            registers: self.registers.clone(),
            hi: self.hi.clone(),
            lo: self.lo.clone(),
            heap_size: self.heap_size,
        }
    }
}