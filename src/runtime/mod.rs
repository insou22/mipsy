#![allow(non_camel_case_types)]
mod display;

use crate::MipsyError;
use std::collections::HashMap;
use crate::{Binary, KDATA_BOT};
use crate::{DATA_BOT, TEXT_BOT, HEAP_BOT, STACK_TOP, KTEXT_BOT};
use crate::error::{
    MipsyResult,
    RuntimeError,
    runtime_error::Uninitialised,
};
use crate::rerr;
use crate::inst::register::Register;
use crate::util::Safe;

const PAGE_SIZE: u32 = 4096;
pub(super) const HI: usize = 32;
pub(super) const LO: usize = 33;


#[allow(dead_code)]
pub struct Runtime {
    timeline: Vec<State>,
    current_state: usize,
    program_len: usize,           // intrinsic
}

#[derive(Clone)]
pub struct State { //       [Safe<u8>; PAGE_SIZE (4096)]
    pages: HashMap<u32, Box<[Safe<u8>]>>,
    pc: u32,
    registers: Vec<Safe<i32>>, // size=34 - 32 registers, $hi, $lo
    register_written: Vec<bool>,
    heap_size: u32,
}

pub type flags = u32;
pub type mode = u32;
pub type len = u32;
pub type fd  = u32;
pub type n_bytes = i32;
pub type void_ptr = Vec<u8>;

pub trait RuntimeHandler {
    fn sys1_print_int   (&mut self, val: i32);
    fn sys2_print_float (&mut self, val: f32);
    fn sys3_print_double(&mut self, val: f64);
    fn sys4_print_string(&mut self, val: String);
    fn sys5_read_int    (&mut self) -> i32;
    fn sys6_read_float  (&mut self) -> f32;
    fn sys7_read_double (&mut self) -> f64;
    fn sys8_read_string (&mut self, max_len: u32) -> String;
    fn sys9_sbrk        (&mut self, val: i32);
    fn sys10_exit       (&mut self);
    fn sys11_print_char (&mut self, val: char);
    fn sys12_read_char  (&mut self) -> char;
    fn sys13_open       (&mut self, path: String, flags: flags, mode: mode) -> fd;
    fn sys14_read       (&mut self, fd: fd, buffer: void_ptr, len: len) -> n_bytes;
    fn sys15_write      (&mut self, fd: fd, buffer: void_ptr, len: len) -> n_bytes;
    fn sys16_close      (&mut self, fd: fd);
    fn sys17_exit_status(&mut self, val: i32);
    fn breakpoint       (&mut self);
}

impl Runtime {
    pub fn new(program: &Binary) -> Self {
        let mut initial_state = 
            State {
                pages: HashMap::new(),
                pc: KTEXT_BOT,
                registers: Default::default(),
                register_written: Default::default(),
                heap_size: 0,
            };
        
        for _ in 0..34 {
            initial_state.registers.push(Safe::Uninitialised);
            initial_state.register_written.push(false);
        }

        let mut text_addr = TEXT_BOT;
        for &word in &program.text {
            initial_state.write_word(text_addr, word);
            text_addr += 4;
        }

        let mut data_addr = DATA_BOT;
        for &byte in &program.data {
            match byte {
                Safe::Valid(byte) => initial_state.write_byte(data_addr, byte),
                Safe::Uninitialised => {}
            }

            data_addr += 1;
        }

        let mut ktext_addr = KTEXT_BOT;
        for &word in &program.ktext {
            initial_state.write_word(ktext_addr, word);
            ktext_addr += 4;
        }

        let mut kdata_addr = KDATA_BOT;
        for &byte in &program.kdata {
            match byte {
                Safe::Valid(byte) => initial_state.write_byte(kdata_addr, byte),
                Safe::Uninitialised => {}
            }

            kdata_addr += 1;
        }

        initial_state.write_ureg(Register::ZERO.to_number() as u32, 0);
        initial_state.write_ureg(Register::SP.to_number() as u32, STACK_TOP);
        initial_state.write_ureg(Register::FP.to_number() as u32, STACK_TOP);
        initial_state.write_ureg(Register::GP.to_number() as u32, HEAP_BOT);

        let runtime = Runtime {
            timeline: vec![initial_state],
            current_state: 0,
            program_len: program.text.len(),
        };

        runtime
    }

    pub fn next_inst(&self) -> MipsyResult<u32> {
        let state = self.state();

        self.state().get_word(state.pc)
    }

    pub fn step<RH>(&mut self, rh: &mut RH) -> MipsyResult<()>
    where
        RH: RuntimeHandler
    {
        let mut state = self.timeline.last().unwrap().clone();

        let inst = state.get_word(state.pc)
                .map_err(|_| MipsyError::Runtime(RuntimeError::NoInstruction(state.pc)))?;
        state.pc += 4;

        self.timeline.push(state);
        self.current_state += 1;

        match self.execute(rh, inst) {
            Err(err) => {
                self.timeline.remove(self.timeline.len() - 1);
                self.current_state -= 1;
                
                Err(err)
            }
            ok => ok,
        }
    }

    pub fn exec_inst<RH>(&mut self, rh: &mut RH, inst: u32) -> MipsyResult<()>
    where
        RH: RuntimeHandler
    {
        self.timeline.push(self.timeline.last().unwrap().clone());
        self.current_state += 1;

        self.execute(rh, inst)?;

        Ok(())
    }

    pub fn back(&mut self) -> bool {
        if self.timeline_len() < 2 {
            return false;
        }

        self.timeline.remove(self.timeline_len() - 1);
        true
    }

    pub fn reset(&mut self) {
        self.timeline.drain(1..);
    }

    pub fn timeline_len(&self) -> usize {
        self.timeline.len()
    }

    pub fn nth_state(&self, n: usize) -> Option<&State> {
        self.timeline.get(n)
    }

    pub fn state(&self) -> &State {
        self.timeline.last().unwrap()
    }

    fn state_mut(&mut self) -> &mut State {
        self.timeline.last_mut().unwrap()
    }

    pub fn prev_state(&self) -> Option<&State> {
        let len = self.timeline.len();

        if len <= 1 {
            None
        } else {
            self.timeline.get(len - 2)
        }
    }

    #[allow(dead_code)]
    fn prev_state_mut(&mut self) -> Option<&mut State> {
        let len = self.timeline.len();

        if len <= 1 {
            None
        } else {
            self.timeline.get_mut(len - 2)
        }
    }

    fn execute<RH>(&mut self, rh: &mut RH, inst: u32) -> MipsyResult<()>
    where
        RH: RuntimeHandler
    {
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
                self.execute_r(rh, funct, rd, rs, rt, shamt)?;
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

        self.state_mut().registers[Register::ZERO.to_number() as usize] = Safe::Valid(0);

        Ok(())
    }

    fn syscall<RH>(&mut self, rh: &mut RH) -> MipsyResult<()>
    where
        RH: RuntimeHandler
    {
        let state = self.state_mut();

        match state.get_reg(Register::V0.to_number() as u32)? {
            1 => { 
                rh.sys1_print_int(state.get_reg(Register::A0.to_number() as u32)?);
            }
            2 => {
                // print_float
                todo!()
            }
            3 => {
                // print_double
                todo!()
            }
            4 => {
                let mut text = String::new();

                let mut pointer = state.get_ureg(Register::A0.to_number() as u32)?;
                loop {
                    let value = state.get_byte(pointer)?;

                    if value == 0 {
                        break;
                    }

                    text.push(value as char);
                    pointer += 1;
                }

                rh.sys4_print_string(text);
            }
            5 => {
                let input = rh.sys5_read_int();
                state.write_reg(Register::V0.to_number() as u32, input);
            }
            6 => {
                // read_float
                todo!()
            },
            7 => {
                // read_double
                todo!()
            },
            8 => {
                let buffer = state.get_ureg(Register::A0.to_number() as u32)?;
                let size   = state.get_ureg(Register::A1.to_number() as u32)?;
                
                let string = rh.sys8_read_string(size);
                for (i, &byte) in (0..size).zip(string.as_bytes()) {
                    state.write_byte(buffer + i, byte);
                }
            },
            9 => {
                let size = state.get_reg(Register::A0.to_number() as u32)?;
                rh.sys9_sbrk(size);

                let state = self.state_mut();
                if size < 0 {
                    let magnitude = ((-1) * size) as u32;
                    if magnitude > state.heap_size {
                        return rerr!(RuntimeError::SbrkNegative);
                    }

                    state.heap_size -= magnitude;
                } else {
                    state.heap_size += size as u32;
                }
            }
            10 => {
                rh.sys10_exit();
            }
            11 => {
                rh.sys11_print_char(state.get_ureg(Register::A0.to_number() as u32)? as u8 as char);
            }
            12 => {
                let char = rh.sys12_read_char();
                state.write_reg(Register::V0.to_number() as u32, char as u8 as u32 as i32);
            }
            13 => {
                // open
                todo!()
            }
            14 => {
                // read
                todo!()
            }
            15 => {
                // write
                todo!()
            }
            16 => {
                // close
                todo!()
            }
            17 => {
                rh.sys17_exit_status(state.get_reg(Register::A0.to_number() as u32).unwrap_or(0));
            }
            _ => {},
        }

        std::io::Write::flush(&mut std::io::stdout()).expect("Error: Couldn't flush stdout?");
        
        Ok(())
    }

    fn execute_r<RH>(&mut self, rh: &mut RH, funct: u32, rd: u32, rs: u32, rt: u32, shamt: u32) -> MipsyResult<()>
    where
        RH: RuntimeHandler
    {
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
            0x08 => { state.pc = state.get_reg(rs)? as u32; },

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
            0x0C => { self.syscall(rh)?; },

            // BREAK
            0x0D => { rh.breakpoint(); },

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

                if rt_val == 0 {
                    return rerr!(RuntimeError::DivisionByZero);
                }

                state.write_lo(rs_val / rt_val);
                state.write_hi(rs_val % rt_val);
            },

            // DIVU $Rs, $Rt
            0x1B => {
                let rs_val = state.get_ureg(rs)?;
                let rt_val = state.get_ureg(rt)?;

                if rt_val == 0 {
                    return rerr!(RuntimeError::DivisionByZero);
                }

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

    fn execute_i(&mut self, opcode: u32, rs: u32, rt: u32, imm: i16) -> MipsyResult<()> {
        let state = self.state_mut();

        let imm_zero_extend = imm as u16 as u32 as i32;
        let imm_sign_extend = imm as i32;

        match opcode {
            // R-Type
            0x00 => unreachable!(),

            0x01 => match rt {
                // BLTZ $Rs, Im
                0x00 => { if state.get_reg(rs)? < 0 { state.branch(imm); } },

                // BGEZ $Rs, Im
                0x01 => { if state.get_reg(rs)? >= 0 { state.branch(imm); } },

                // Error
                _ => todo!(),
            },

            // Unused
            0x02 => {},
            
            // Unused
            0x03 => {},
            
            // BEQ  $Rs, $Rt, Im
            0x04 => { if state.get_reg(rs)? == state.get_reg(rt)? { state.branch(imm); } },
            
            // BNE  $Rs, $Rt, Im
            0x05 => { if state.get_reg(rs)? != state.get_reg(rt)? { state.branch(imm); } },
            
            // BLEZ $Rs, Im
            0x06 => { if state.get_reg(rs)? <= 0 { state.branch(imm); } },
            
            // BGTZ $Rs, Im
            0x07 => { if state.get_reg(rs)? > 0 { state.branch(imm); } },
            
            // ADDI $Rt, $Rs, Im
            0x08 => { state.write_reg(rt, checked_add_imm(state.get_reg(rs)?, imm)?) },
            
            // ADDIU $Rt, $Rs, Im
            0x09 => { state.write_reg(rt, state.get_reg(rs)?.wrapping_add(imm_sign_extend)) },
            
            // SLTI $Rt, $Rs, Im
            0x0A => { if state.get_reg(rs)? < imm_sign_extend { state.write_reg(rt, 1); } else { state.write_reg(rt, 0); } },
            
            // SLTIU $Rt, $Rs, Im
            0x0B => { if (state.get_reg(rs)? as u32) < imm_sign_extend as u32 { state.write_reg(rt, 1); } else { state.write_reg(rt, 0); } },
            
            // ANDI $Rt, $Rs, Im
            0x0C => { state.write_reg(rt, state.get_reg(rs)? & imm_zero_extend); },
            
            // ORI  $Rt, $Rs, Im
            0x0D => { state.write_reg(rt, state.get_reg(rs)? | imm_zero_extend); },
            
            // XORI $Rt, $Rs, Im
            0x0E => { state.write_reg(rt, state.get_reg(rs)? ^ imm_zero_extend); },
            
            // LUI  $Rt, Im
            0x0F => { state.write_reg(rt, imm_zero_extend << 16 as i32); },
            
            // Unused
            0x10..=0x1F => {},
            
            // LB   $Rt, Im($Rs)
            0x20 => { state.load_byte(rt, state.get_ureg(rs)?.wrapping_add(imm_sign_extend as u32)); },
            
            // LH   $Rt, Im($Rs)
            0x21 => { state.load_half(rt, state.get_ureg(rs)?.wrapping_add(imm_sign_extend as u32)); },
            
            // Unused
            0x22 => {},
            
            // LW   $Rt, Im($Rs)
            0x23 => { state.load_word(rt, state.get_ureg(rs)?.wrapping_add(imm_sign_extend as u32)); },
            
            // LBU  $Rt, Im($Rs)
            0x24 => { state.load_ubyte(rt, state.get_ureg(rs)?.wrapping_add(imm_sign_extend as u32)); },
            
            // LHU  $Rt, Im($Rs)
            0x25 => { state.load_uhalf(rt, state.get_ureg(rs)?.wrapping_add(imm_sign_extend as u32)); },
            
            // Unused
            0x26 => {},
            
            // Unused
            0x27 => {},
            
            // SB   $Rt, Im($Rs)
            0x28 => { state.store_byte(rt, state.get_ureg(rs)?.wrapping_add(imm_sign_extend as u32)); },
            
            // SH   $Rt, Im($Rs)
            0x29 => { state.store_half(rt, state.get_ureg(rs)?.wrapping_add(imm_sign_extend as u32)); },
            
            // Unused
            0x2A => {},
            
            // SW   $Rt, Im($Rs)
            0x2B => { state.store_word(rt, state.get_ureg(rs)?.wrapping_add(imm_sign_extend as u32)); },
            
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
            0x31 => { todo!() },
            
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
            0x39 => { todo!() },
            
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

    fn execute_j(&mut self, opcode: u32, target: u32) -> MipsyResult<()> {
        let state = self.state_mut();

        match opcode {
            // J    addr
            0x02 => { 
                state.pc = (state.pc & 0xF000_0000) | (target << 2);
            },

            // JAL  addr
            0x03 => { 
                state.write_ureg(Register::RA.to_number() as u32, state.pc);
                state.pc = (state.pc & 0xF000_0000) | (target << 2);
            },

            _ => unreachable!(),
        }
        Ok(())
    }
}

impl State {
    pub fn get_pc(&self) -> u32 {
        self.pc
    }

    fn branch(&mut self, imm: i16) {
        // println!("Branching with imm = {} --  pc 0x{:08x} ==> 0x{:08x}", imm, self.pc, self.pc.wrapping_add(((imm as i32 - 1) * 4) as u32));
        self.pc = self.pc.wrapping_add(((imm as i32 - 1) * 4) as u32);
    }

    fn load_word(&mut self, reg: u32, addr: u32) {
        match self.get_word(addr) {
            Ok(w)  => self.write_ureg(reg, w),
            Err(_) => self.reset_reg(reg),
        }
    }

    fn load_half(&mut self, reg: u32, addr: u32) {
        match self.get_half(addr) {
            Ok(h)  => self.write_reg(reg, h as i16 as i32),
            Err(_) => self.reset_reg(reg),
        }
    }

    fn load_byte(&mut self, reg: u32, addr: u32) {
        match self.get_byte(addr) {
            Ok(b)  => self.write_reg(reg, b as i8 as i32),
            Err(_) => self.reset_reg(reg),
        }
    }

    fn load_uhalf(&mut self, reg: u32, addr: u32) {
        match self.get_half(addr) {
            Ok(h)  => self.write_ureg(reg, h as u32),
            Err(_) => self.reset_reg(reg),
        }
    }

    fn load_ubyte(&mut self, reg: u32, addr: u32) {
        match self.get_byte(addr) {
            Ok(b)  => self.write_ureg(reg, b as u32),
            Err(_) => self.reset_reg(reg),
        }
    }

    fn store_word(&mut self, reg: u32, addr: u32) {
        match self.get_reg(reg) {
            Ok(val) => self.write_word(addr, val as u32),
            Err(_)  => self.reset_word(addr),
        }
    }

    fn store_half(&mut self, reg: u32, addr: u32) {
        match self.get_reg(reg) {
            Ok(val) => self.write_half(addr, val as u16),
            Err(_)  => self.reset_half(addr),
        }
    }

    fn store_byte(&mut self, reg: u32, addr: u32) {
        match self.get_reg(reg) {
            Ok(val) => self.write_byte(addr, val as u8),
            Err(_)  => self.reset_byte(addr),
        }
    }

    fn reset_reg(&mut self, reg: u32) {
        self.register_written[reg as usize] = true;
        self.registers[reg as usize] = Safe::Uninitialised;
    }

    pub fn get_reg(&self, reg: u32) -> MipsyResult<i32> {
        match self.registers[reg as usize] {
            Safe::Valid(reg) => Ok(reg),
            Safe::Uninitialised => rerr!(RuntimeError::Uninitialised(Uninitialised::Register(reg))),
        }
    }

    fn get_ureg(&self, reg: u32) -> MipsyResult<u32> {
        self.get_reg(reg).map(|x| x as u32)
    }

    #[allow(unreachable_code)]
    pub fn write_reg(&mut self, reg: u32, value: i32) {
        if reg == 0 && value != 0 {
            // TODO: Warning - cannot write to $zero
            return;
        }

        self.registers[reg as usize] = Safe::Valid(value);
        self.register_written[reg as usize] = true;
    }

    fn write_ureg(&mut self, reg: u32, value: u32) {
        self.registers[reg as usize] = Safe::Valid(value as i32);
        self.register_written[reg as usize] = true;
    }

    pub fn get_hi(&self) -> MipsyResult<i32> {
        match self.registers[HI] {
            Safe::Valid(val) => Ok(val),
            Safe::Uninitialised => rerr!(RuntimeError::Uninitialised(Uninitialised::Hi)),
        }
    }

    pub fn get_lo(&self) -> MipsyResult<i32> {
        match self.registers[LO] {
            Safe::Valid(val) => Ok(val),
            Safe::Uninitialised => rerr!(RuntimeError::Uninitialised(Uninitialised::Lo)),
        }
    }

    pub fn write_hi(&mut self, value: i32) {
        self.registers[HI] = Safe::Valid(value);
        self.register_written[HI] = true;
    }

    pub fn write_lo(&mut self, value: i32) {
        self.registers[LO] = Safe::Valid(value);
        self.register_written[LO] = true;
    }

    fn write_uhi(&mut self, value: u32) {
        self.registers[HI] = Safe::Valid(value as i32);
        self.register_written[HI] = true;
    }

    fn write_ulo(&mut self, value: u32) {
        self.registers[LO] = Safe::Valid(value as i32);
        self.register_written[LO] = true;
    }

    pub fn get_word(&self, address: u32) -> MipsyResult<u32> {
        let (b1, b2, b3, b4) = (|| {
            let b1 = self.get_byte(address)?     as u32;
            let b2 = self.get_byte(address + 1)? as u32;
            let b3 = self.get_byte(address + 2)? as u32;
            let b4 = self.get_byte(address + 3)? as u32;
        
            Ok((b1, b2, b3, b4))
        })().map_err(|_: MipsyError| MipsyError::Runtime(RuntimeError::Uninitialised(Uninitialised::Word(address))))?;

        Ok(b1 | (b2 << 8) | (b3 << 16) | (b4 << 24))
    }

    pub fn get_half(&self, address: u32) -> MipsyResult<u16> {
        let (b1, b2) = (|| {
            let b1 = self.get_byte(address)?     as u16;
            let b2 = self.get_byte(address + 1)? as u16;

            Ok((b1, b2))
        })().map_err(|_: MipsyError| MipsyError::Runtime(RuntimeError::Uninitialised(Uninitialised::Half(address))))?;

        Ok(b1 | (b2 << 8))
    }

    pub fn get_byte(&self, address: u32) -> MipsyResult<u8> {
        (|| {
            let page = self.get_page(address)?;
            let offset = Self::offset_in_page(address);
    
            page[offset as usize].as_option().copied()
        })().ok_or(MipsyError::Runtime(RuntimeError::Uninitialised(Uninitialised::Byte(address))))
    }

    pub fn write_word(&mut self, address: u32, word: u32) {
        let page = self.get_page_or_create(address);
        let offset = Self::offset_in_page(address);

        // Little endian
        page[offset as usize]     = Safe::Valid((word & 0x000000FF) as u8);
        page[offset as usize + 1] = Safe::Valid(((word & 0x0000FF00) >> 8) as u8);
        page[offset as usize + 2] = Safe::Valid(((word & 0x00FF0000) >> 16) as u8);
        page[offset as usize + 3] = Safe::Valid(((word & 0xFF000000) >> 24) as u8);
    }

    pub fn write_half(&mut self, address: u32, half: u16) {
        let page = self.get_page_or_create(address);
        let offset = Self::offset_in_page(address);

        // Little endian
        page[offset as usize]     = Safe::Valid((half & 0x00FF) as u8);
        page[offset as usize + 1] = Safe::Valid(((half & 0xFF00) >> 8) as u8);
    }

    pub fn write_byte(&mut self, address: u32, byte: u8) {
        let page = self.get_page_or_create(address);
        let offset = Self::offset_in_page(address);

        page[offset as usize] = Safe::Valid(byte);
    }

    pub fn reset_word(&mut self, address: u32) {
        let page = self.get_page_or_create(address);
        let offset = Self::offset_in_page(address);

        page[offset as usize]     = Safe::Uninitialised;
        page[offset as usize + 1] = Safe::Uninitialised;
        page[offset as usize + 2] = Safe::Uninitialised;
        page[offset as usize + 3] = Safe::Uninitialised;

    }

    pub fn reset_half(&mut self, address: u32) {
        let page = self.get_page_or_create(address);
        let offset = Self::offset_in_page(address);

        page[offset as usize]     = Safe::Uninitialised;
        page[offset as usize + 1] = Safe::Uninitialised;
    }

    pub fn reset_byte(&mut self, address: u32) {
        let page = self.get_page_or_create(address);
        let offset = Self::offset_in_page(address);

        page[offset as usize] = Safe::Uninitialised;
    }

    pub fn is_register_written(&self, reg: u32) -> bool {
        self.register_written[reg as usize]
    }

    pub fn is_hi_written(&self) -> bool {
        self.register_written[HI]
    }

    pub fn is_lo_written(&self) -> bool {
        self.register_written[LO]
    }

    fn get_page_or_create(&mut self, address: u32) -> &mut [Safe<u8>] {
        let base_addr = Self::addr_to_page_base_addr(address);
        
        self.pages.entry(base_addr).or_insert_with(|| Box::new([Default::default(); PAGE_SIZE as usize]))
    }

    fn get_page(&self, address: u32) -> Option<&[Safe<u8>]> {
        let base_addr = Self::addr_to_page_base_addr(address);
        self.pages.get(&base_addr).map(|page| &**page)
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

fn checked_add(x: i32, y: i32) -> MipsyResult<i32> {
    match x.checked_add(y) {
        Some(z) => Ok(z),
        None => rerr!(RuntimeError::IntegerOverflow),
    }
}

fn checked_add_imm(x: i32, y: i16) -> MipsyResult<i32> {
    match x.checked_add(y as i32) {
        Some(z) => Ok(z),
        None => rerr!(RuntimeError::IntegerOverflow),
    }
}

fn checked_sub(x: i32, y: i32) -> MipsyResult<i32> {
    match x.checked_sub(y) {
        Some(z) => Ok(z),
        None => rerr!(RuntimeError::IntegerOverflow),
    }
}
