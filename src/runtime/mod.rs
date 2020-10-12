use std::collections::HashMap;
use crate::compile::context::Program;
use crate::compile::compiler::{DATA_BOT, TEXT_BOT, HEAP_BOT, STACK_TOP};
use crate::error::RSpimResult;
use crate::error::RuntimeError;
use crate::rerr;
use crate::inst::register::Register;

pub const PAGE_SIZE: u32 = 4096;

pub struct Runtime {
    timeline: Vec<State>,
    program_len: usize, // intrinsic
    labels: HashMap<String, u32>, // intrinsic
}

#[derive(Clone)]
pub struct State {
    pages: HashMap<u32, [Safe<u8>; PAGE_SIZE as usize]>,
    pc: u32,
    registers: [Safe<i32>; 32],
    hi: Safe<i32>,
    lo: Safe<i32>,
}

#[derive(Copy)]
pub enum Safe<T> {
    Valid(T),
    Uninitialised,
}

impl<T> Safe<T> {
    fn as_option(self) -> Option<T> {
        match self {
            Self::Valid(t) => Some(t),
            Self::Uninitialised => None,
        }
    }
}

impl<T> Clone for Safe<T>
where T: Clone {
    fn clone(&self) -> Self {
        match self {
            Self::Valid(t) => Self::Valid(t.clone()),
            Self::Uninitialised => Self::Uninitialised,
        }
    }
}

impl<T> Default for Safe<T> {
    fn default() -> Self {
        Self::Uninitialised
    }
}

impl Runtime {
    pub fn new(program: &Program) -> Self {
        let mut initial_state = 
            State {
                pages: HashMap::new(),
                pc: TEXT_BOT,
                registers: Default::default(),
                hi: Default::default(),
                lo: Default::default(),
            };
        
        let mut text_addr = TEXT_BOT;
        for &word in &program.text {
            initial_state.write_word(text_addr, word);
            text_addr += 4;
        }

        let mut data_addr = DATA_BOT;
        for &byte in &program.data {
            initial_state.write_byte(data_addr, byte);
            data_addr += 4;
        }

        initial_state.write_ureg(Register::ZERO.to_number() as u32, 0);
        initial_state.write_ureg(Register::SP.to_number() as u32, STACK_TOP);
        initial_state.write_ureg(Register::GP.to_number() as u32, HEAP_BOT);

        let runtime = Runtime {
            timeline: vec![initial_state],
            program_len: program.text.len(),
            labels: program.labels.clone(),
        };

        runtime
    }

    pub fn step(&mut self) -> RSpimResult<()> {
        self.timeline.push(self.timeline.last().unwrap().clone());
        let state = self.timeline.last_mut().unwrap();

        let inst = state.get_word(state.pc)?;
        state.pc += 4;

        self.execute(inst)?;

        Ok(())
    }

    fn state(&self) -> &State {
        self.timeline.last().unwrap()
    }

    fn state_mut(&mut self) -> &mut State {
        self.timeline.last_mut().unwrap()
    }

    fn execute(&mut self, inst: u32) -> RSpimResult<()> {
        let opcode =  inst >> 26;
        let rs     = (inst >> 21) & 0x1F;
        let rt     = (inst >> 16) & 0x1F;
        let rd     = (inst >> 11) & 0x1F;
        let shamt  = (inst >>  6) & 0x1F;
        let funct  =  inst & 0x3F;
        let imm    = (inst & 0xFFFF) as i16;
        let addr   =  inst & 0x3FFFFFF;

        match opcode {
            0 => {
                // R-Type
                self.execute_r(funct, rd, rs, rt, shamt)?;
            }
            0b000010 | 0b000011 => {
                // J-Type
                self.execute_j(opcode, addr)?;
            }
            _ => {
                // I-Type
                self.execute_i(opcode, rs, rt, imm)?;
            }
        }

        Ok(())
    }

    fn syscall(&mut self) -> RSpimResult<()> {
        Ok(())
    }

    fn execute_r(&mut self, funct: u32, rd: u32, rs: u32, rt: u32, shamt: u32) -> RSpimResult<()> {
        let state = self.state_mut();

        match funct {
            // SLL  $Rd, $Rt, Sa
            0x00 => { state.write_reg(rd, (state.get_ureg(rt)? << shamt) as i32); },

            // Unused
            0x01 => {},

            // SRL  $Rd, $Rt, Sa
            0x02 => { state.write_reg(rd, (state.get_ureg(rt)? >> shamt) as i32); },

            // SRA  $Rd, $Rt, Sa
            0x03 => { state.write_reg(rd, state.get_reg(rt)? >> shamt); },

            // SLLV $Rd, $Rt, $Rs
            0x04 => { state.write_reg(rd, (state.get_ureg(rt)? << state.get_ureg(rs)?) as i32); },

            // Unused
            0x05 => {},

            // SRLV $Rd, $Rt, $Rs
            0x06 => { state.write_reg(rd, (state.get_ureg(rt)? >> state.get_ureg(rs)?) as i32); },

            // SRAV $Rd, $Rt, $Rs
            0x07 => { state.write_reg(rd, state.get_reg(rt)? >> state.get_reg(rs)?); },

            // JR   $Rs
            0x08 => { state.pc = state.get_reg(rs)? as u32 },

            // JALR $Rs
            0x09 => { 
                state.write_ureg(Register::RA.to_number() as u32, state.pc); 
                state.pc = state.get_ureg(rs)?;
            },
            
            // Unused
            0x0A => {},

            // Unused
            0x0B => {},

            // SYSCALL
            0x0C => { self.syscall()?; },

            // BREAK
            0x0D => { todo!(); },

            // Unused
            0x0E => {},

            // Unused
            0x0F => {},

            // MFHI $Rd
            0x10 => { state.write_reg(rd, state.get_hi()?); },

            // MTHI $Rs
            0x11 => { state.write_hi(state.get_reg(rs)?); },

            // MFLO $Rd
            0x12 => { state.write_reg(rd, state.get_lo()?); },

            // MTLO $Rs
            0x13 => { state.write_lo(state.get_reg(rs)?); },

            // Unused
            0x14 => {},

            // Unused
            0x15 => {},

            // Unused
            0x16 => {},

            // Unused
            0x17 => {},

            // MULT $Rs, $Rt
            0x18 => {
                let rs_val = state.get_reg(rs)?;
                let rt_val = state.get_reg(rt)?;

                let result = (rs_val as i64 * rt_val as i64) as u64;
                state.write_uhi((result >> 32) as u32);
                state.write_ulo((result & 0xFFFF_FFFF) as u32);
            },

            // MULTU $Rs, $Rt
            0x19 => {
                let rs_val = state.get_reg(rs)?;
                let rt_val = state.get_reg(rt)?;

                let result = rs_val as u64 * rt_val as u64;
                state.write_uhi((result >> 32) as u32);
                state.write_ulo((result & 0xFFFF_FFFF) as u32);
            },

            // DIV  $Rs, $Rt
            0x1A => {
                let rs_val = state.get_reg(rs)?;
                let rt_val = state.get_reg(rt)?;

                state.write_lo(rs_val / rt_val);
                state.write_hi(rs_val % rt_val);
            },

            // DIVU $Rs, $Rt
            0x1B => {
                let rs_val = state.get_ureg(rs)?;
                let rt_val = state.get_ureg(rt)?;

                state.write_ulo(rs_val / rt_val);
                state.write_uhi(rs_val % rt_val);
            },

            // Unused
            0x1C => {},

            // Unused
            0x1D => {},

            // Unused
            0x1E => {},

            // Unused
            0x1F => {},

            // ADD  $Rd, $Rs, $Rt
            0x20 => { state.write_reg(rd, checked_add(state.get_reg(rs)?, state.get_reg(rt)?)?); },

            // ADDU $Rd, $Rs, $Rt
            0x21 => { state.write_reg(rd, state.get_reg(rs)?.wrapping_add(state.get_reg(rt)?)); },

            // SUB  $Rd, $Rs, $Rt
            0x22 => { state.write_reg(rd, checked_sub(state.get_reg(rs)?, state.get_reg(rt)?)?); },

            // SUBU $Rd, $Rs, $Rt
            0x23 => { state.write_reg(rd, state.get_reg(rs)?.wrapping_sub(state.get_reg(rt)?)); },

            // AND  $Rd, $Rs, $Rt
            0x24 => { state.write_reg(rd, state.get_reg(rs)? & state.get_reg(rt)?); },

            // OR   $Rd, $Rs, $Rt
            0x25 => { state.write_reg(rd, state.get_reg(rs)? | state.get_reg(rt)?); },

            // XOR  $Rd, $Rs, $Rt
            0x26 => { state.write_reg(rd, state.get_reg(rs)? ^ state.get_reg(rt)?); },

            // NOR  $Rd, $Rs, $Rt
            0x27 => { state.write_reg(rd, ! (state.get_reg(rs)? | state.get_reg(rt)?)); },

            // Unused
            0x28 => {},

            // Unused
            0x29 => {},

            // SLT  $Rd, $Rs, $Rt
            0x2A => { state.write_reg(rd, if state.get_reg(rs)? < state.get_reg(rt)? { 1 } else { 0 } ); },

            // SLTU $Rd, $Rs, $Rt
            0x2B => { state.write_reg(rd, if state.get_ureg(rs)? < state.get_ureg(rt)? { 1 } else { 0 } ); },

            // Unused
            0x2C..=0x3F => {},

            // Doesn't fit in 6 bits
            _ => unreachable!(),
        }

        Ok(())
    }

    fn execute_i(&mut self, opcode: u32, rs: u32, rt: u32, imm: i16) -> RSpimResult<()> {
        match opcode {
            // R-Type
            0x00 => unreachable!(),

            0x01 => match rt {
                // BLTZ $Rs, Im
                0x00 => {},

                // BGEZ $Rs, Im
                0x01 => {},

                // Error
                _ => todo!(),
            },

            // Unused
            0x02 => {},
            
            // Unused
            0x03 => {},
            
            // BEQ  $Rs, $Rt, Im
            0x04 => {},
            
            // BNE  $Rs, $Rt, Im
            0x05 => {},
            
            // BLEZ $Rs, Im
            0x06 => {},
            
            // BGTZ $Rs, Im
            0x07 => {},
            
            // ADDI $Rt, $Rs, Im
            0x08 => {},
            
            // ADDIU $Rt, $Rs, Im
            0x09 => {},
            
            // SLTI $Rt, $Rs, Im
            0x0A => {},
            
            // SLTIU $Rt, $Rs, Im
            0x0B => {},
            
            // ANDI $Rt, $Rs, Im
            0x0C => {},
            
            // ORI  $Rt, $Rs, Im
            0x0D => {},
            
            // XORI $Rt, $Rs, Im
            0x0E => {},
            
            // LUI  $Rt, Im
            0x0F => {},
            
            // Unused
            0x10..=0x1F => {},
            
            // LB   $Rt, Im($Rs)
            0x20 => {},
            
            // LH   $Rt, Im($Rs)
            0x21 => {},
            
            // Unused
            0x22 => {},
            
            // LW   $Rt, Im($Rs)
            0x23 => {},
            
            // LBU  $Rt, Im($Rs)
            0x24 => {},
            
            // LHU  $Rt, Im($Rs)
            0x25 => {},
            
            // Unused
            0x26 => {},
            
            // Unused
            0x27 => {},
            
            // SB   $Rt, Im($Rs)
            0x28 => {},
            
            // SH   $Rt, Im($Rs)
            0x29 => {},
            
            // Unused
            0x2A => {},
            
            // SW   $Rt, Im($Rs)
            0x2B => {},
            
            // Unused
            0x2C => {},
            
            // Unused
            0x2D => {},
            
            // Unused
            0x2E => {},
            
            // Unused
            0x2F => {},
            
            // Unused
            0x30 => {},
            
            // LWC1 $Rt, Im($Rs)
            0x31 => {},
            
            // Unused
            0x32 => {},
            
            // Unused
            0x33 => {},
            
            // Unused
            0x34 => {},
            
            // Unused
            0x35 => {},
            
            // Unused
            0x36 => {},
            
            // Unused
            0x37 => {},
            
            // Unused
            0x38 => {},
            
            // SWC1 $Rt, Im($Rs)
            0x39 => {},
            
            // Unused
            0x3A => {},
            
            // Unused
            0x3B => {},
            
            // Unused
            0x3C => {},
            
            // Unused
            0x3D => {},
            
            // Unused
            0x3E => {},
            
            // Unused
            0x3F => {},

            // Doesn't fit in 6 bits
            _ => unreachable!(),
        }

        Ok(())
    }

    fn execute_j(&mut self, opcode: u32, target: u32) -> RSpimResult<()> {
        match opcode {
            // J    addr
            0x02 => {},

            // JAL  addr
            0x03 => {},

            _ => unreachable!(),
        }
        Ok(())
    }
}

impl State {
    fn get_reg(&self, reg: u32) -> RSpimResult<i32> {
        match self.registers[reg as usize] {
            Safe::Valid(reg) => Ok(reg),
            Safe::Uninitialised => rerr!(RuntimeError::UninitializedRegister(reg)),
        }
    }

    fn get_ureg(&self, reg: u32) -> RSpimResult<u32> {
        self.get_reg(reg).map(|x| x as u32)
    }

    #[allow(unreachable_code)]
    fn write_reg(&mut self, reg: u32, value: i32) {
        if reg == 0 && value != 0 {
            todo!("warning: cannot write to $ZERO");
            return;
        }

        self.registers[reg as usize] = Safe::Valid(value);
    }

    fn write_ureg(&mut self, reg: u32, value: u32) {
        self.registers[reg as usize] = Safe::Valid(value as i32);
    }

    fn get_hi(&self) -> RSpimResult<i32> {
        match self.hi {
            Safe::Valid(val) => Ok(val),
            Safe::Uninitialised => rerr!(RuntimeError::UninitializedHi),
        }
    }

    fn get_lo(&self) -> RSpimResult<i32> {
        match self.lo {
            Safe::Valid(val) => Ok(val),
            Safe::Uninitialised => rerr!(RuntimeError::UninitializedLo),
        }
    }

    fn write_hi(&mut self, value: i32) {
        self.hi = Safe::Valid(value);
    }

    fn write_lo(&mut self, value: i32) {
        self.lo = Safe::Valid(value);
    }

    fn write_uhi(&mut self, value: u32) {
        self.hi = Safe::Valid(value as i32);
    }

    fn write_ulo(&mut self, value: u32) {
        self.lo = Safe::Valid(value as i32);
    }

    fn get_word(&self, address: u32) -> RSpimResult<u32> {
        let b1 = self.get_byte(address + 0)? as u32;
        let b2 = self.get_byte(address + 1)? as u32;
        let b3 = self.get_byte(address + 2)? as u32;
        let b4 = self.get_byte(address + 3)? as u32;

        Ok(b1 | (b2 << 8) | (b3 << 16) | (b4 << 24))
    }

    fn get_half(&self, address: u32) -> RSpimResult<u16> {
        let b1 = self.get_byte(address + 0)? as u16;
        let b2 = self.get_byte(address + 1)? as u16;

        Ok(b1 | (b2 << 8))
    }

    fn get_byte(&self, address: u32) -> RSpimResult<u8> {
        let page = self.get_page(address)?;
        let offset = Self::offset_in_page(address);

        let value = match page[offset as usize] {
            Safe::Valid(value) => value,
            Safe::Uninitialised => return rerr!(RuntimeError::UninitializedMemory(address)),
        };

        Ok(value)
    }

    fn write_word(&mut self, address: u32, word: u32) {
        let page = self.get_page_or_create(address);
        let offset = Self::offset_in_page(address);

        // Little endian
        page[offset as usize + 0] = Safe::Valid((word & 0x000000FF) as u8);
        page[offset as usize + 1] = Safe::Valid((word & 0x0000FF00) as u8);
        page[offset as usize + 2] = Safe::Valid((word & 0x00FF0000) as u8);
        page[offset as usize + 3] = Safe::Valid((word & 0xFF000000) as u8);
    }

    fn write_half(&mut self, address: u32, half: u16) {
        let page = self.get_page_or_create(address);
        let offset = Self::offset_in_page(address);

        // Little endian
        page[offset as usize + 0] = Safe::Valid((half & 0x00FF) as u8);
        page[offset as usize + 1] = Safe::Valid((half & 0xFF00) as u8);
    }

    fn write_byte(&mut self, address: u32, byte: u8) {
        let page = self.get_page_or_create(address);
        let offset = Self::offset_in_page(address);

        page[offset as usize] = Safe::Valid(byte);
    }

    fn get_page_or_create(&'_ mut self, address: u32) -> &'_ mut [Safe<u8>; PAGE_SIZE as usize] {
        let base_addr = Self::addr_to_page_base_addr(address);
        let page = self.pages.entry(base_addr).or_insert([Default::default(); PAGE_SIZE as usize]);

        page
    }

    fn get_page(&'_ self, address: u32) -> RSpimResult<&'_ [Safe<u8>; PAGE_SIZE as usize]> {
        let base_addr = Self::addr_to_page_base_addr(address);
        let page = self.pages.get(&base_addr);

        match page {
            Some(page) => Ok(page),
            None => rerr!(RuntimeError::PageNotExist(address))
        }
    }

    fn get_page_mut(&'_ mut self, address: u32) -> RSpimResult<&'_ mut [Safe<u8>; PAGE_SIZE as usize]> {
        let base_addr = Self::addr_to_page_base_addr(address);
        let page = self.pages.get_mut(&base_addr);

        match page {
            Some(page) => Ok(page),
            None => rerr!(RuntimeError::PageNotExist(address))
        }
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
}

fn checked_add(x: i32, y: i32) -> RSpimResult<i32> {
    match x.checked_add(y) {
        Some(z) => Ok(z),
        None => rerr!(RuntimeError::IntegerOverflow),
    }
}

fn checked_sub(x: i32, y: i32) -> RSpimResult<i32> {
    match x.checked_sub(y) {
        Some(z) => Ok(z),
        None => rerr!(RuntimeError::IntegerOverflow),
    }
}
