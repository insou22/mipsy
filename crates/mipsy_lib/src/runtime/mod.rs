pub mod state;

pub use self::state::State;

use std::collections::HashMap;
use crate::{Binary, DATA_BOT, HEAP_BOT, KDATA_BOT, KTEXT_BOT, MipsyError, MipsyResult, Register, RuntimeError, STACK_PTR, Safe, TEXT_BOT, Uninitialised, error::runtime::{AlignmentRequirement, InvalidReason, Error}, compile::GLOBAL_PTR};
use self::state::Timeline;

use crate::util::{get_segment, Segment};

pub const NUL:  u8  = 0;
pub const NULL: u32 = 0;
pub const PAGE_SIZE: usize = 64;

pub const SYS1_PRINT_INT:    i32 = 1;
pub const SYS2_PRINT_FLOAT:  i32 = 2;
pub const SYS3_PRINT_DOUBLE: i32 = 3;
pub const SYS4_PRINT_STRING: i32 = 4;
pub const SYS5_READ_INT:     i32 = 5;
pub const SYS6_READ_FLOAT:   i32 = 6;
pub const SYS7_READ_DOUBLE:  i32 = 7;
pub const SYS8_READ_STRING:  i32 = 8;
pub const SYS9_SBRK:         i32 = 9;
pub const SYS10_EXIT:        i32 = 10;
pub const SYS11_PRINT_CHAR:  i32 = 11;
pub const SYS12_READ_CHAR:   i32 = 12;
pub const SYS13_OPEN:        i32 = 13;
pub const SYS14_READ:        i32 = 14;
pub const SYS15_WRITE:       i32 = 15;
pub const SYS16_CLOSE:       i32 = 16;
pub const SYS17_EXIT_STATUS: i32 = 17;

pub const SPECIAL:  u32 = 0b000000;
pub const SPECIAL2: u32 = 0b011100;
pub const SPECIAL3: u32 = 0b011111;

macro_rules! try_owned_self {
    ($self:ident, $res:expr) => {
        match $res {
            Ok(res) => res,
            Err(err) => return Err(($self, err)),
        }
    };
}

pub struct Runtime {
    timeline: Timeline,
}

impl Runtime {
    pub fn timeline(&self) -> &Timeline {
        &self.timeline
    }

    pub fn timeline_mut(&mut self) -> &mut Timeline {
        &mut self.timeline
    }

    pub fn step(mut self) -> Result<SteppedRuntime, (Runtime, MipsyError)> {
        let state = self.timeline.state();
        let segment = get_segment(state.pc());
        match segment {
            Segment::Text | Segment::KText => {}
            _ => {
                let addr = state.pc();
                return Err((self, MipsyError::Runtime(RuntimeError::new(Error::SegmentationFault { addr }))));
            }
        }
        let inst = match state.read_mem_word(state.pc()) {
            Ok(inst) => inst,
            Err(_) => {
                let addr = state.pc();
                return Err((self, MipsyError::Runtime(RuntimeError::new(Error::UnknownInstruction { addr }))));
            }
        };

        let state = self.timeline.push_next_state();
        state.set_pc(state.pc() + 4);

        match self.execute_in_current_state(inst) {
            Err((mut new_self, err)) => {
                new_self.timeline.pop_last_state();

                Err((new_self, err))
            }
            ok => ok,
        }
    }

    pub fn exec_inst(mut self, opcode: u32) -> Result<SteppedRuntime, (Runtime, MipsyError)> {
        self.timeline.push_next_state();

        match self.execute_in_current_state(opcode) {
            Err((mut new_self, err)) => {
                new_self.timeline.pop_last_state();

                Err((new_self, err))
            }
            ok => ok,
        }
    }

    pub fn next_inst(&self) -> MipsyResult<u32> {
        let state = self.timeline().state();

        self.timeline().state().read_mem_word(state.pc())
    }

    pub fn next_inst_may_guard(&self) -> MipsyResult<bool> {
        let inst = self.next_inst()?;

        let opcode =  inst >> 26;
        let funct  =  inst  & 0x3F;

        Ok(
            opcode == 0 && (funct == 0xC || funct == 0xD)
        )
    }

    fn execute_in_current_state(mut self, inst: u32) -> Result<SteppedRuntime, (Runtime, MipsyError)>
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
            SPECIAL | SPECIAL2 | SPECIAL3 => {
                // R-Type
                self.execute_r(opcode, funct, rd, rs, rt, shamt)
            }
            0b000010 | 0b000011 => {
                // J-Type
                self.execute_j(opcode, addr);

                Ok(Ok(self))
            }
            _ => {
                // I-Type
                self.execute_i(opcode, rs, rt, imm)
            }
        }
    }

    // remove when floating point syscalls are finished
    #[allow(unreachable_code)]
    fn syscall(mut self) -> Result<RuntimeSyscallGuard, (Runtime, MipsyError)> {
        let syscall = try_owned_self!(self, self.timeline.state().read_register(Register::V0.to_u32()));
    
        Ok(
            match syscall {
                SYS1_PRINT_INT => {
                    let value = try_owned_self!(self, self.timeline.state().read_register(Register::A0.to_u32()));

                    RuntimeSyscallGuard::PrintInt(
                        PrintIntArgs {
                            value
                        },
                        self
                    )
                }
                SYS2_PRINT_FLOAT => {
                    return Err((self, MipsyError::Runtime(RuntimeError::new(Error::InvalidSyscall { syscall, reason: InvalidReason::Unimplemented }))));
                }
                // RuntimeSyscallGuard::PrintFloat(
                //     PrintFloatArgs {
                //         value: todo!(),
                //     },
                //     self
                // ),
                SYS3_PRINT_DOUBLE => {
                    return Err((self, MipsyError::Runtime(RuntimeError::new(Error::InvalidSyscall { syscall, reason: InvalidReason::Unimplemented }))));
                }
                // RuntimeSyscallGuard::PrintDouble(
                //     PrintDoubleArgs {
                //         value: todo!(),
                //     },
                //     self
                // ),
                SYS4_PRINT_STRING => {
                    let value = try_owned_self!(self, self.timeline.state().read_mem_string(
                        try_owned_self!(self, self.timeline.state().read_register(Register::A0.to_u32())) as _
                    ));

                    RuntimeSyscallGuard::PrintString(
                        PrintStringArgs {
                            value,
                        },
                        self
                    )
                }
                SYS5_READ_INT => RuntimeSyscallGuard::ReadInt(
                    Box::new(move |value| {
                        self.timeline.state_mut().write_register(Register::V0.to_u32(), value);
                        self
                    })
                ),
                SYS6_READ_FLOAT => {
                    return Err((self, MipsyError::Runtime(RuntimeError::new(Error::InvalidSyscall { syscall, reason: InvalidReason::Unimplemented }))));
                }
                SYS7_READ_DOUBLE => {
                    return Err((self, MipsyError::Runtime(RuntimeError::new(Error::InvalidSyscall { syscall, reason: InvalidReason::Unimplemented }))));
                }
                SYS8_READ_STRING => {
                    let buf = try_owned_self!(self, self.timeline.state().read_register(Register::A0.to_u32())) as u32;
                    let len = try_owned_self!(self, self.timeline.state().read_register(Register::A1.to_u32())) as _;

                    RuntimeSyscallGuard::ReadString(
                        ReadStringArgs {
                            max_len: len,
                        },
                        Box::new(move |mut string| {
                            if len > 0 {
                                let max_bytes = (len - 1) as usize;

                                if string.len() >= max_bytes {
                                    string.resize(max_bytes, 0);
                                }

                                string.push(0);
                                
                                for (i, byte) in string.into_iter().enumerate() {
                                    // if there's a segmentation fault, we just don't end up writing the data
                                    let _ = self.timeline.state_mut().write_mem_byte(buf + i as u32, byte);
                                }
                            }

                            self
                        })
                    )
                }
                SYS9_SBRK => {
                    let bytes = try_owned_self!(self, self.timeline.state().read_register(Register::A0.to_u32()));
                    let heap_size = self.timeline.state().heap_size();

                    self.timeline.state_mut().write_register(Register::V0.to_u32(), (HEAP_BOT + heap_size) as _);

                    if bytes > 0 {
                        self.timeline.state_mut().set_heap_size(heap_size.saturating_add(bytes as _));
                    } else if bytes < 0 {
                        self.timeline.state_mut().set_heap_size(heap_size.saturating_sub(bytes.abs() as _));
                    }

                    RuntimeSyscallGuard::Sbrk(
                        SbrkArgs {
                            bytes,
                        },
                        self,
                    )
                }
                SYS10_EXIT => RuntimeSyscallGuard::Exit(
                    self
                ),
                SYS11_PRINT_CHAR => RuntimeSyscallGuard::PrintChar(
                    PrintCharArgs {
                        value: try_owned_self!(self, self.timeline.state().read_register(Register::A0.to_u32())) as _,
                    },
                    self
                ),
                SYS12_READ_CHAR => RuntimeSyscallGuard::ReadChar(
                    Box::new(move |value| {
                        self.timeline.state_mut().write_register(Register::V0.to_u32(), value as _);
                        self
                    })
                ),
                SYS13_OPEN => RuntimeSyscallGuard::Open(
                    OpenArgs {
                        path: try_owned_self!(self, self.timeline.state().read_mem_string(
                            try_owned_self!(self, self.timeline.state().read_register(Register::A0.to_u32())) as _
                        )),
                        flags: try_owned_self!(self, self.timeline.state().read_register(Register::A1.to_u32())) as _,
                        mode:  try_owned_self!(self, self.timeline.state().read_register(Register::A2.to_u32())) as _,
                    },
                    Box::new(move |fd| {
                        self.timeline.state_mut().write_register(Register::V0.to_u32(), fd as _);
                        self
                    })
                ),
                SYS14_READ => {
                    let fd  = try_owned_self!(self, self.timeline.state().read_register(Register::A0.to_u32())) as _;
                    let buf = try_owned_self!(self, self.timeline.state().read_register(Register::A1.to_u32())) as u32;
                    let len = try_owned_self!(self, self.timeline.state().read_register(Register::A2.to_u32())) as _;
                
                    RuntimeSyscallGuard::Read(
                        ReadArgs {
                            fd,
                            len,
                        },
                        Box::new(move |(n_bytes, bytes)| {
                            let len = (len as usize).min(bytes.len());
                
                            bytes[..len].iter().enumerate().for_each(|(i, byte)| {
                                // if there's a segmentation fault, we just don't end up writing the data
                                let _ = self.timeline.state_mut().write_mem_byte(buf + i as u32, *byte);
                            });
                            self.timeline.state_mut().write_register(Register::V0.to_u32(), n_bytes);
                
                            self
                        })
                    )
                }
                SYS15_WRITE =>{
                    let fd  = try_owned_self!(self, self.timeline.state().read_register(Register::A0.to_u32())) as _;
                    let buf = try_owned_self!(self, self.timeline.state().read_register(Register::A1.to_u32())) as _;
                    let len = try_owned_self!(self, self.timeline.state().read_register(Register::A2.to_u32())) as _;
                
                    RuntimeSyscallGuard::Write(
                        WriteArgs {
                            fd,
                            buf: try_owned_self!(self, self.timeline.state().read_mem_bytes(buf, len)),
                        },
                        Box::new(move |written| {
                            self.timeline.state_mut().write_register(Register::V0.to_u32(), written as _);
                
                            self
                        })
                    )
                }
                SYS16_CLOSE => RuntimeSyscallGuard::Close(
                    CloseArgs {
                        fd: try_owned_self!(self, self.timeline.state().read_register(Register::A0.to_u32())) as _,
                    },
                    Box::new(move |status| {
                        self.timeline.state_mut().write_register(Register::V0.to_u32(), status as _);
                        self
                    })
                ),
                SYS17_EXIT_STATUS => RuntimeSyscallGuard::ExitStatus(
                    ExitStatusArgs {
                        exit_code: self.timeline.state().read_register_uninit(Register::A0.to_u32()).into_option().unwrap_or(0) as _,
                    },
                    self,
                ),
                _ => {
                    return Err((self, MipsyError::Runtime(RuntimeError::new(Error::InvalidSyscall { syscall, reason: InvalidReason::Unknown }))));
                }
            }
        )
    }

    fn execute_r(mut self, special: u32, funct: u32, rd: u32, rs: u32, rt: u32, shamt: u32) -> Result<SteppedRuntime, (Runtime, MipsyError)>
    {
        let state = self.timeline.state();

        match (special, funct) {
            // SYSCALL
            (SPECIAL, 0x0C) => {
                Ok(Err(self.syscall()?))
            }

            // BREAK
            (SPECIAL, 0x0D) => {
                Ok(Err(RuntimeSyscallGuard::Breakpoint(self)))
            }

            // TGE  $Rs, $Rt
            (SPECIAL, 0x30) => {
                if try_owned_self!(self, state.read_register(rs)) >= try_owned_self!(self, state.read_register(rt)) {
                    Ok(Err(RuntimeSyscallGuard::Trap(self)))
                } else {
                    Ok(Ok(self))
                }
            }

            // TGEU $Rs, $Rt
            (SPECIAL, 0x31) => {
                if try_owned_self!(self, state.read_register(rs)) as u32 >= try_owned_self!(self, state.read_register(rt)) as u32 {
                    Ok(Err(RuntimeSyscallGuard::Trap(self)))
                } else {
                    Ok(Ok(self))
                }
            }

            // TLT  $Rs, $Rt
            (SPECIAL, 0x32) => {
                if try_owned_self!(self, state.read_register(rs)) < try_owned_self!(self, state.read_register(rt)) {
                    Ok(Err(RuntimeSyscallGuard::Trap(self)))
                } else {
                    Ok(Ok(self))
                }
            }

            // TLTU $Rs, $Rt
            (SPECIAL, 0x33) => {
                if (try_owned_self!(self, state.read_register(rs)) as u32) < try_owned_self!(self, state.read_register(rt)) as u32 {
                    Ok(Err(RuntimeSyscallGuard::Trap(self)))
                } else {
                    Ok(Ok(self))
                }
            }

            // TEQ  $Rs, $Rt
            (SPECIAL, 0x34) => {
                if try_owned_self!(self, state.read_register(rs)) == try_owned_self!(self, state.read_register(rt)) {
                    Ok(Err(RuntimeSyscallGuard::Trap(self)))
                } else {
                    Ok(Ok(self))
                }
            }

            // TNE  $Rs, $Rt
            (SPECIAL, 0x36) => {
                if try_owned_self!(self, state.read_register(rs)) != try_owned_self!(self, state.read_register(rt)) {
                    Ok(Err(RuntimeSyscallGuard::Trap(self)))
                } else {
                    Ok(Ok(self))
                }
            }

            _ => {
                try_owned_self!(self, self.execute_non_trapping_r(special, funct, rd, rs, rt, shamt));
                Ok(SteppedRuntime::Ok(self))
            }
        }
    }

    fn execute_non_trapping_r(&mut self, special: u32, funct: u32, rd: u32, rs: u32, rt: u32, shamt: u32) -> MipsyResult<()> {
        let state = self.timeline.state_mut();

        match special {
            SPECIAL => {
                match funct {
                    // SLL  $Rd, $Rt, Sa
                    0x00 => { state.write_register(rd, ((state.read_register(rt)? as u32) << shamt) as i32); },
        
                    // Unused
                    0x01 => {},
        
                    0x02 => {
                        match rs {
                            // SRL  $Rd, $Rt, Sa
                            0x00 => { state.write_register(rd, ((state.read_register(rt)? as u32) >> shamt) as i32); }
                            
                            // ROTR $Rd, $Rt, Sa
                            0x01 => { state.write_register(rd, ((state.read_register(rt)? as u32).rotate_right(shamt)) as i32); }

                            _ => todo!(),
                        }
                    },
        
                    // SRA  $Rd, $Rt, Sa
                    0x03 => { state.write_register(rd, state.read_register(rt)? >> shamt); },
        
                    // SLLV $Rd, $Rt, $Rs
                    0x04 => { state.write_register(rd, ((state.read_register(rt)? as u32) << state.read_register(rs)?) as i32); },
        
                    // Unused
                    0x05 => {},
        
                    0x06 => {
                        match shamt {
                            // SRLV $Rd, $Rt, $Rs
                            0x00 => { state.write_register(rd, ((state.read_register(rt)? as u32) >> state.read_register(rs)?) as i32); },
                            
                            // ROTRV $Rd, $Rt, $Rs
                            0x01 => { state.write_register(rd, ((state.read_register(rt)? as u32).rotate_right(state.read_register(rs)? as u32)) as i32); },

                            _ => todo!(),
                        }
                    }
        
                    // SRAV $Rd, $Rt, $Rs
                    0x07 => { state.write_register(rd, state.read_register(rt)? >> state.read_register(rs)?); },
        
                    // JR   $Rs
                    0x08 => { state.set_pc(state.read_register(rs)? as u32); },
        
                    // JALR $Rs
                    0x09 => { 
                        state.write_register(rd, state.pc() as _); 
                        state.set_pc(state.read_register(rs)? as _);
                    },
                    
                    // MOVZ $Rd, $Rs, $Rt
                    0x0A => {
                        if state.read_register(rt)? == 0 {
                            state.write_register(rd, state.read_register(rs)?);
                        }
                    },

                    // MOVN $Rd, $Rs, $Rt
                    0x0B => {
                        if state.read_register(rt)? != 0 {
                            state.write_register(rd, state.read_register(rs)?);
                        }
                    },
        
                    // SYSCALL
                    0x0C => unreachable!("covered above"),
        
                    // BREAK
                    0x0D => unreachable!("covered above"),
        
                    // Unused
                    0x0E => {},
        
                    // Unused
                    0x0F => {},
        
                    0x10 => match shamt {
                        // MFHI $Rd
                        0x00 => { state.write_register(rd, state.read_hi()?); }

                        // CLZ $Rd, $Rs
                        0x01 => { state.write_register(rd, state.read_register(rs)?.leading_zeros() as i32); }

                        _ => todo!(),
                    }
        
                    0x11 => match shamt {
                        // MTHI $Rs
                        0x00 => { state.write_hi(state.read_register(rs)?); }
                        
                        // CLO $Rd, $Rs
                        0x01 => { state.write_register(rd, state.read_register(rs)?.leading_ones() as i32); }

                        _ => todo!(),
                    }
        
                    // MFLO $Rd
                    0x12 => { state.write_register(rd, state.read_lo()?); },
        
                    // MTLO $Rs
                    0x13 => { state.write_lo(state.read_register(rs)?); },
        
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
                        let rs_val = state.read_register(rs)?;
                        let rt_val = state.read_register(rt)?;
        
                        let result = (rs_val as i64 * rt_val as i64) as u64;
                        state.write_hi((result >> 32) as _);
                        state.write_lo((result & 0xFFFF_FFFF) as _);
                    },
        
                    // MULTU $Rs, $Rt
                    0x19 => {
                        let rs_val = state.read_register(rs)? as u32;
                        let rt_val = state.read_register(rt)? as u32;
        
                        let result = rs_val as u64 * rt_val as u64;
                        state.write_hi((result >> 32) as _);
                        state.write_lo((result & 0xFFFF_FFFF) as _);
                    },
        
                    // DIV  $Rs, $Rt
                    0x1A => {
                        let rs_val = state.read_register(rs)?;
                        let rt_val = state.read_register(rt)?;
        
                        if rt_val == 0 {
                            return Err(MipsyError::Runtime(RuntimeError::new(Error::DivisionByZero)));
                        }
        
                        state.write_lo(rs_val / rt_val);
                        state.write_hi(rs_val % rt_val);
                    },
        
                    // DIVU $Rs, $Rt
                    0x1B => {
                        let rs_val = state.read_register(rs)? as u32;
                        let rt_val = state.read_register(rt)? as u32;
        
                        if rt_val == 0 {
                            return Err(MipsyError::Runtime(RuntimeError::new(Error::DivisionByZero)));
                        }
        
                        state.write_lo((rs_val / rt_val) as i32);
                        state.write_hi((rs_val % rt_val) as i32);
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
                    0x20 => { state.write_register(rd, checked_add(state.read_register(rs)?, state.read_register(rt)?)?); },
        
                    // ADDU $Rd, $Rs, $Rt
                    0x21 => { state.write_register(rd, state.read_register(rs)?.wrapping_add(state.read_register(rt)?)); },
        
                    // SUB  $Rd, $Rs, $Rt
                    0x22 => { state.write_register(rd, checked_sub(state.read_register(rs)?, state.read_register(rt)?)?); },
        
                    // SUBU $Rd, $Rs, $Rt
                    0x23 => { state.write_register(rd, state.read_register(rs)?.wrapping_sub(state.read_register(rt)?)); },
        
                    // AND  $Rd, $Rs, $Rt
                    0x24 => { state.write_register(rd, state.read_register(rs)? & state.read_register(rt)?); },
        
                    // OR   $Rd, $Rs, $Rt
                    0x25 => { state.write_register(rd, state.read_register(rs)? | state.read_register(rt)?); },
        
                    // XOR  $Rd, $Rs, $Rt
                    0x26 => { state.write_register(rd, state.read_register(rs)? ^ state.read_register(rt)?); },
        
                    // NOR  $Rd, $Rs, $Rt
                    0x27 => { state.write_register(rd, ! (state.read_register(rs)? | state.read_register(rt)?)); },
        
                    // Unused
                    0x28 => {},
        
                    // Unused
                    0x29 => {},
        
                    // SLT  $Rd, $Rs, $Rt
                    0x2A => { state.write_register(rd, if state.read_register(rs)? < state.read_register(rt)? { 1 } else { 0 } ); },
        
                    // SLTU $Rd, $Rs, $Rt
                    0x2B => { state.write_register(rd, if state.read_register(rs)? < state.read_register(rt)? { 1 } else { 0 } ); },
        
                    // Unused
                    0x2C..=0x3F => {},
        
                    // Doesn't fit in 6 bits
                    _ => unreachable!(),
                }        
            }
            SPECIAL2 => {
                match funct {
                    // MADD
                    0x00 => {
                        let rs_val = state.read_register(rs)?;
                        let rt_val = state.read_register(rt)?;
        
                        let original = ((state.read_hi()? as u64) << 32) | state.read_lo()? as u64;
                        let result = original + (rs_val as i64 * rt_val as i64) as u64;

                        state.write_hi((result >> 32) as _);
                        state.write_lo((result & 0xFFFF_FFFF) as _);
                    }
                    
                    // MADDU
                    0x01 => {
                        let rs_val = state.read_register(rs)?;
                        let rt_val = state.read_register(rt)?;
        
                        let original = ((state.read_hi()? as u64) << 32) | state.read_lo()? as u64;
                        let result = original + rs_val as u64 * rt_val as u64;

                        state.write_hi((result >> 32) as _);
                        state.write_lo((result & 0xFFFF_FFFF) as _);
                    }
                    
                    // MSUB
                    0x04 => {
                        let rs_val = state.read_register(rs)?;
                        let rt_val = state.read_register(rt)?;
        
                        let original = ((state.read_hi()? as u64) << 32) | state.read_lo()? as u64;
                        let result = original - (rs_val as i64 * rt_val as i64) as u64;

                        state.write_hi((result >> 32) as _);
                        state.write_lo((result & 0xFFFF_FFFF) as _);
                    }
                    
                    // MSUBU
                    0x05 => {
                        let rs_val = state.read_register(rs)?;
                        let rt_val = state.read_register(rt)?;
        
                        let original = ((state.read_hi()? as u64) << 32) | state.read_lo()? as u64;
                        let result = original - rs_val as u64 * rt_val as u64;

                        state.write_hi((result >> 32) as _);
                        state.write_lo((result & 0xFFFF_FFFF) as _);
                    }

                    // Unimplemented
                    _ => {}
                }
            }
            SPECIAL3 => {
                match funct {
                    0x20 => {
                        match shamt {
                            // WSBH $Rd, $Rt
                            0x02 => {
                                let rt_val = state.read_register(rt)? as u32;
                                let bottom_half =  rt_val        as u16;
                                let top_half    = (rt_val >> 16) as u16;

                                let bottom_half_swapped = bottom_half.swap_bytes() as u32;
                                let top_half_swapped    = top_half.swap_bytes()    as u32;

                                let result = bottom_half_swapped | (top_half_swapped << 16);

                                state.write_register(rd, result as i32);
                            },

                            // SEB  $Rd, $Rt
                            0x10 => { state.write_register(rd, (state.read_register(rt)? as u8 ).extend_sign()); },
                            
                            // SEH  $Rd, $Rt
                            0x18 => { state.write_register(rd, (state.read_register(rt)? as u16).extend_sign()); },
                        
                            _ => todo!(),
                        }
                    }

                    _ => todo!(),
                }
            }
            _ => unreachable!("special can only be SPECIAL, SPECIAL2, or SPECIAL3"),
        }

        Ok(())
    }


    fn execute_i(mut self, opcode: u32, rs: u32, rt: u32, imm: i16) -> Result<SteppedRuntime, (Runtime, MipsyError)> {
        let state = self.timeline.state();

        match (opcode, rt) {
            // TGEI
            (0x01, 0x08) => {
                if try_owned_self!(self, state.read_register(rs)) >= imm.extend_sign() {
                    Ok(Err(RuntimeSyscallGuard::Trap(self)))
                } else {
                    Ok(Ok(self))
                }
            }

            // TGEIU
            (0x01, 0x09) => {
                if try_owned_self!(self, state.read_register(rs)) as u32 >= imm.extend_sign() as u32 {
                    Ok(Err(RuntimeSyscallGuard::Trap(self)))
                } else {
                    Ok(Ok(self))
                }
            }

            // TLTI
            (0x01, 0x0A) => {
                if try_owned_self!(self, state.read_register(rs)) < imm.extend_sign() {
                    Ok(Err(RuntimeSyscallGuard::Trap(self)))
                } else {
                    Ok(Ok(self))
                }
            }

            // TLTIU
            (0x01, 0x0B) => {
                if (try_owned_self!(self, state.read_register(rs)) as u32) < imm.extend_sign() as u32 {
                    Ok(Err(RuntimeSyscallGuard::Trap(self)))
                } else {
                    Ok(Ok(self))
                }
            }

            // TEQI
            (0x01, 0x0C) => {
                if try_owned_self!(self, state.read_register(rs)) == imm.extend_sign() {
                    Ok(Err(RuntimeSyscallGuard::Trap(self)))
                } else {
                    Ok(Ok(self))
                }
            }

            // TNEI
            (0x01, 0x0E) => {
                if try_owned_self!(self, state.read_register(rs)) != imm.extend_sign() {
                    Ok(Err(RuntimeSyscallGuard::Trap(self)))
                } else {
                    Ok(Ok(self))
                }
            }

            _ => {
                try_owned_self!(self, self.execute_non_trapping_i(opcode, rs, rt, imm));
                Ok(SteppedRuntime::Ok(self))
            }
        }
    }


    fn execute_non_trapping_i(&mut self, opcode: u32, rs: u32, rt: u32, imm: i16) -> MipsyResult<()> {
        let state = self.timeline.state_mut();

        let imm_zero_extend = imm as u16 as u32 as i32;
        let imm_sign_extend = imm as i32;

        match opcode {
            // R-Type
            0x00 => unreachable!(),

            0x01 => match rt {
                // BLTZ $Rs, Im
                0x00 => { if state.read_register(rs)? < 0 { state.branch(imm); } },

                // BGEZ $Rs, Im
                0x01 => { if state.read_register(rs)? >= 0 { state.branch(imm); } },

                // BLTZAL $Rs, Im
                0x10 => {
                    if state.read_register(rs)? < 0 {
                        state.write_register(Register::Ra.to_number() as u32, state.pc() as _);
                        state.branch(imm);
                    }
                }

                // BGEZAL $Rs, Im
                0x11 => {
                    if state.read_register(rs)? >= 0 {
                        state.write_register(Register::Ra.to_number() as u32, state.pc() as _);
                        state.branch(imm);
                    }
                }

                // Error
                _ => todo!(),
            },

            // Unused
            0x02 => {},
            
            // Unused
            0x03 => {},
            
            // BEQ  $Rs, $Rt, Im
            0x04 => { if state.read_register(rs)? == state.read_register(rt)? { state.branch(imm); } },
            
            // BNE  $Rs, $Rt, Im
            0x05 => { if state.read_register(rs)? != state.read_register(rt)? { state.branch(imm); } },
            
            // BLEZ $Rs, Im
            0x06 => { if state.read_register(rs)? <= 0 { state.branch(imm); } },
            
            // BGTZ $Rs, Im
            0x07 => { if state.read_register(rs)? > 0 { state.branch(imm); } },
            
            // ADDI $Rt, $Rs, Im
            0x08 => { state.write_register(rt, checked_add(state.read_register(rs)?, imm_sign_extend)?) },
            
            // ADDIU $Rt, $Rs, Im
            0x09 => { state.write_register(rt, state.read_register(rs)?.wrapping_add(imm_sign_extend)) },
            
            // SLTI $Rt, $Rs, Im
            0x0A => { if state.read_register(rs)? < imm_sign_extend { state.write_register(rt, 1); } else { state.write_register(rt, 0); } },
            
            // SLTIU $Rt, $Rs, Im
            0x0B => { if (state.read_register(rs)? as u32) < imm_sign_extend as u32 { state.write_register(rt, 1); } else { state.write_register(rt, 0); } },
            
            // ANDI $Rt, $Rs, Im
            0x0C => { state.write_register(rt, state.read_register(rs)? & imm_zero_extend); },
            
            // ORI  $Rt, $Rs, Im
            0x0D => { state.write_register(rt, state.read_register(rs)? | imm_zero_extend); },
            
            // XORI $Rt, $Rs, Im
            0x0E => { state.write_register(rt, state.read_register(rs)? ^ imm_zero_extend); },
            
            // LUI  $Rt, Im
            0x0F => { state.write_register(rt, imm_zero_extend << 16); },
            
            // Unused
            0x10..=0x1F => {},
            
            // LB   $Rt, Im($Rs)
            0x20 => { state.write_register_uninit(rt, state.read_mem_byte_uninit(state.read_register(rs)?.wrapping_add(imm_sign_extend) as _)?.extend_sign()); },
            
            // LH   $Rt, Im($Rs)
            0x21 => {
                let addr = state.read_register(rs)?.wrapping_add(imm_sign_extend) as _;

                if addr % 2 != 0 {
                    return Err(MipsyError::Runtime(RuntimeError::new(Error::UnalignedAccess { addr, alignment_requirement: AlignmentRequirement::Half })));
                }

                state.write_register_uninit(rt, state.read_mem_half_uninit(addr)?.extend_sign()); 
            },
            
            // LWL  $Rt, Im($Rs)
            0x22 => {
                todo!();
            },
            
            // LW   $Rt, Im($Rs)
            0x23 => {
                let addr = state.read_register(rs)?.wrapping_add(imm_sign_extend) as _;

                if addr % 4 != 0 {
                    return Err(MipsyError::Runtime(RuntimeError::new(Error::UnalignedAccess { addr, alignment_requirement: AlignmentRequirement::Word })));
                }

                state.write_register_uninit(rt, state.read_mem_word_uninit(addr)?.extend_sign());
            },
            
            // LBU  $Rt, Im($Rs)
            0x24 => { state.write_register_uninit(rt, state.read_mem_byte_uninit(state.read_register(rs)?.wrapping_add(imm_sign_extend) as _)?.extend_zero()); },
            
            // LHU  $Rt, Im($Rs)
            0x25 => {
                let addr = state.read_register(rs)?.wrapping_add(imm_sign_extend) as _;

                if addr % 2 != 0 {
                    return Err(MipsyError::Runtime(RuntimeError::new(Error::UnalignedAccess { addr, alignment_requirement: AlignmentRequirement::Half })));
                }

                state.write_register_uninit(rt, state.read_mem_half_uninit(addr)?.extend_zero());
            },
            
            // LWR  $Rt, Im($Rs)
            0x26 => {
                todo!();
            },
            
            // Unused
            0x27 => {},
            
            // SB   $Rt, Im($Rs)
            0x28 => { state.write_mem_byte_uninit(state.read_register(rs)?.wrapping_add(imm_sign_extend) as _, state.read_register_uninit(rt).truncate())?; },
            
            // SH   $Rt, Im($Rs)
            0x29 => {
                let addr = state.read_register(rs)?.wrapping_add(imm_sign_extend) as _;

                if addr % 2 != 0 {
                    return Err(MipsyError::Runtime(RuntimeError::new(Error::UnalignedAccess { addr, alignment_requirement: AlignmentRequirement::Half })));
                }

                state.write_mem_half_uninit(addr, state.read_register_uninit(rt).truncate())?;
            },
            
            // Unused
            0x2A => {},
            
            // SW   $Rt, Im($Rs)
            0x2B => {
                let addr = state.read_register(rs)?.wrapping_add(imm_sign_extend) as _;

                if addr % 4 != 0 {
                    return Err(MipsyError::Runtime(RuntimeError::new(Error::UnalignedAccess { addr, alignment_requirement: AlignmentRequirement::Word })));
                }

                state.write_mem_word_uninit(addr, state.read_register_uninit(rt).truncate())?;
            },
            
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

    fn execute_j(&mut self, opcode: u32, target: u32) {
        let state = self.timeline.state_mut();

        match opcode {
            // J    addr
            0x02 => {
                state.set_pc((state.pc() & 0xF000_0000) | (target << 2))
            },

            // JAL  addr
            0x03 => { 
                state.write_register(Register::Ra.to_number() as u32, state.pc() as _);
                state.set_pc((state.pc() & 0xF000_0000) | (target << 2));
            },

            _ => unreachable!(),
        }
    }

}

pub type SteppedRuntime = Result<Runtime, RuntimeSyscallGuard>;

pub enum RuntimeSyscallGuard {
    PrintInt   (PrintIntArgs,    Runtime),
    PrintFloat (PrintFloatArgs,  Runtime),
    PrintDouble(PrintDoubleArgs, Runtime),
    PrintString(PrintStringArgs, Runtime),
    ReadInt    (                Box<dyn FnOnce(i32)     -> Runtime>),
    ReadFloat  (                Box<dyn FnOnce(f32)     -> Runtime>),
    ReadDouble (                Box<dyn FnOnce(f64)     -> Runtime>),
    ReadString (ReadStringArgs, Box<dyn FnOnce(Vec<u8>) -> Runtime>),
    Sbrk       (SbrkArgs, Runtime),
    Exit       (Runtime),
    PrintChar  (PrintCharArgs, Runtime),
    ReadChar   (           Box<dyn FnOnce(u8)             -> Runtime>),
    Open       (OpenArgs,  Box<dyn FnOnce(i32)            -> Runtime>),
    Read       (ReadArgs,  Box<dyn FnOnce((i32, Vec<u8>)) -> Runtime>),
    Write      (WriteArgs, Box<dyn FnOnce(i32)            -> Runtime>),
    Close      (CloseArgs, Box<dyn FnOnce(i32)            -> Runtime>),
    ExitStatus (ExitStatusArgs, Runtime),

    // other
    Breakpoint     (Runtime),
    Trap           (Runtime),
}

pub struct PrintIntArgs {
    pub value: i32,
}

pub struct PrintFloatArgs {
    pub value: f32,
}

pub struct PrintDoubleArgs {
    pub value: f64,
}

pub struct PrintStringArgs {
    pub value: Vec<u8>,
}

pub struct ReadStringArgs {
    pub max_len: u32,
}

pub struct SbrkArgs {
    pub bytes: i32,
}

pub struct PrintCharArgs {
    pub value: u8,
}

pub struct OpenArgs {
    pub path: Vec<u8>,
    pub flags: u32,
    pub mode: u32,
}

pub struct ReadArgs {
    pub fd: u32,
    pub len: u32,
}

pub struct WriteArgs {
    pub fd: u32,
    pub buf: Vec<u8>,
}

pub struct CloseArgs {
    pub fd: u32,
}

pub struct ExitStatusArgs {
    pub exit_code: i32,
}

pub(self) trait SafeToUninitResult {
    type Output;

    fn to_result(&self, value_type: Uninitialised) -> MipsyResult<Self::Output>;
}

impl<T: Copy> SafeToUninitResult for Safe<T> {
    type Output = T;

    fn to_result(&self, value_type: Uninitialised) -> MipsyResult<Self::Output> {
        match self {
            Safe::Valid(value)  => Ok(*value),
            Safe::Uninitialised => Err(
                MipsyError::Runtime(
                    RuntimeError::new(
                        Error::Uninitialised { value: value_type }
                    )
                )
            ),
        }
    }
}

impl<T: Copy> SafeToUninitResult for Option<T> {
    type Output = T;

    fn to_result(&self, value_type: Uninitialised) -> MipsyResult<Self::Output> {
        match self {
            Some(value)  => Ok(*value),
            None => Err(
                MipsyError::Runtime(
                    RuntimeError::new(
                        Error::Uninitialised { value: value_type }
                    )
                )
            ),
        }
    }
}

trait ExtendSign {
    type Output;

    fn extend_sign(self) -> Self::Output;
}

trait ExtendZero {
    type Output;

    fn extend_zero(self) -> Self::Output;
}

trait Truncate<T> {
    fn truncate(self) -> T;
}

impl ExtendSign for u8 {
    type Output = i32;

    fn extend_sign(self) -> Self::Output {
        self as i8 as _
    }
}

impl ExtendSign for u16 {
    type Output = i32;

    fn extend_sign(self) -> Self::Output {
        self as i16 as _
    }
}

impl ExtendSign for u32 {
    type Output = i32;

    fn extend_sign(self) -> Self::Output {
        self as _
    }
}

impl ExtendSign for i8 {
    type Output = i32;

    fn extend_sign(self) -> Self::Output {
        self as _
    }
}

impl ExtendSign for i16 {
    type Output = i32;

    fn extend_sign(self) -> Self::Output {
        self as _
    }
}

impl ExtendSign for i32 {
    type Output = i32;

    fn extend_sign(self) -> Self::Output {
        self
    }
}

impl ExtendZero for u8 {
    type Output = i32;

    fn extend_zero(self) -> Self::Output {
        self as u8 as _
    }
}

impl ExtendZero for u16 {
    type Output = i32;

    fn extend_zero(self) -> Self::Output {
        self as u16 as _
    }
}

impl ExtendZero for u32 {
    type Output = i32;

    fn extend_zero(self) -> Self::Output {
        self as _
    }
}

impl Truncate<u8> for i32 {
    fn truncate(self) -> u8 {
        self as _
    }
}

impl Truncate<u16> for i32 {
    fn truncate(self) -> u16 {
        self as _
    }
}

impl Truncate<u32> for i32 {
    fn truncate(self) -> u32 {
        self as _
    }
}

impl ExtendSign for Safe<u8> {
    type Output = Safe<i32>;

    fn extend_sign(self) -> Self::Output {
        match self {
            Safe::Valid(value)  => Safe::Valid(value.extend_sign()),
            Safe::Uninitialised => Safe::Uninitialised,
        }
    }
}

impl ExtendSign for Safe<u16> {
    type Output = Safe<i32>;

    fn extend_sign(self) -> Self::Output {
        match self {
            Safe::Valid(value)  => Safe::Valid(value.extend_sign()),
            Safe::Uninitialised => Safe::Uninitialised,
        }
    }
}

impl ExtendSign for Safe<u32> {
    type Output = Safe<i32>;

    fn extend_sign(self) -> Self::Output {
        match self {
            Safe::Valid(value)  => Safe::Valid(value.extend_sign()),
            Safe::Uninitialised => Safe::Uninitialised,
        }
    }
}

impl ExtendZero for Safe<u8> {
    type Output = Safe<i32>;

    fn extend_zero(self) -> Self::Output {
        match self {
            Safe::Valid(value)  => Safe::Valid(value.extend_zero()),
            Safe::Uninitialised => Safe::Uninitialised,
        }
    }
}

impl ExtendZero for Safe<u16> {
    type Output = Safe<i32>;

    fn extend_zero(self) -> Self::Output {
        match self {
            Safe::Valid(value)  => Safe::Valid(value.extend_zero()),
            Safe::Uninitialised => Safe::Uninitialised,
        }
    }
}

impl ExtendZero for Safe<u32> {
    type Output = Safe<i32>;

    fn extend_zero(self) -> Self::Output {
        match self {
            Safe::Valid(value)  => Safe::Valid(value.extend_zero()),
            Safe::Uninitialised => Safe::Uninitialised,
        }
    }
}

impl Truncate<Safe<u8>> for Safe<i32> {
    fn truncate(self) -> Safe<u8> {
        match self {
            Safe::Valid(value)  => Safe::Valid(value.truncate()),
            Safe::Uninitialised => Safe::Uninitialised,
        }
    }
}

impl Truncate<Safe<u16>> for Safe<i32> {
    fn truncate(self) -> Safe<u16> {
        match self {
            Safe::Valid(value)  => Safe::Valid(value.truncate()),
            Safe::Uninitialised => Safe::Uninitialised,
        }
    }
}

impl Truncate<Safe<u32>> for Safe<i32> {
    fn truncate(self) -> Safe<u32> {
        match self {
            Safe::Valid(value)  => Safe::Valid(value.truncate()),
            Safe::Uninitialised => Safe::Uninitialised,
        }
    }
}

impl Runtime {
    pub fn new(program: &Binary, args: &[&str]) -> Self {
        let mut initial_state = 
            State {
                pages: HashMap::new(),
                pc: KTEXT_BOT,
                heap_size: 0,
                registers: Default::default(),
                write_marker: 0,
                hi: Default::default(),
                lo: Default::default(),
            };

        let mut text_addr = TEXT_BOT;
        for &byte in &program.text {
            initial_state.write_mem_byte_uninit(text_addr, byte).unwrap();
            text_addr += 1;
        }

        let mut data_addr = DATA_BOT;
        for &byte in &program.data {
            match byte {
                Safe::Valid(byte) => initial_state.write_mem_byte(data_addr, byte).unwrap(),
                Safe::Uninitialised => {}
            }

            data_addr += 1;
        }

        let mut ktext_addr = KTEXT_BOT;
        for &byte in &program.ktext {
            initial_state.write_mem_byte_uninit(ktext_addr, byte).unwrap();
            ktext_addr += 1;
        }

        let mut kdata_addr = KDATA_BOT;
        for &byte in &program.kdata {
            match byte {
                Safe::Valid(byte) => initial_state.write_mem_byte(kdata_addr, byte).unwrap(),
                Safe::Uninitialised => {}
            }

            kdata_addr += 1;
        }

        initial_state.registers[Register::Zero.to_number() as usize] = Safe::Valid(0);
        initial_state.write_register(Register::Sp.to_number() as _, STACK_PTR  as _);
        initial_state.write_register(Register::Fp.to_number() as _, STACK_PTR  as _);
        initial_state.write_register(Register::Gp.to_number() as _, GLOBAL_PTR as _);

        Self::include_args(&mut initial_state, args);

        Self {
            timeline: Timeline::new(initial_state),
        }
    }

    fn include_args(state: &mut State, args: &[&str]) {
        if args.is_empty() {
            state.write_register(Register::A0.to_u32(), 0);
            state.write_register(Register::A1.to_u32(), NULL as _);

            return;
        }

        let total_strings_len = args.iter()
            .fold(0, |len, string| len + string.bytes().count() + 1)
            as u32;

        // allocate total_strings_len on the stack
        let strings_stack_addr = STACK_PTR - total_strings_len;

        // and then 4-byte align it
        let strings_stack_addr = strings_stack_addr - (strings_stack_addr % 4);

        let total_char_star_stars_len = (args.len() + 1) * 4;
        let total_char_star_stars_len = total_char_star_stars_len as u32;

        let char_star_star_addr = strings_stack_addr - total_char_star_stars_len;

        state.write_register(Register::A0.to_u32(),  args.len() as _);
        state.write_register(Register::A1.to_u32(),  char_star_star_addr as _);
        state.write_register(Register::Sp.to_u32(), (char_star_star_addr - 4) as _);

        {
            let mut string_addr = strings_stack_addr;
            let mut star_addr   = char_star_star_addr;

            for &arg in args {
                state.write_mem_word(star_addr, string_addr).unwrap();

                for byte in arg.bytes() {
                    state.write_mem_byte(string_addr, byte).unwrap();

                    string_addr += 1;
                }

                // null terminator
                state.write_mem_byte(string_addr, NUL).unwrap();
                string_addr += 1;

                star_addr += 4;
            }

            state.write_mem_word(star_addr, NULL).unwrap();
        }
    }
}

fn checked_add(x: i32, y: i32) -> MipsyResult<i32> {
    match x.checked_add(y) {
        Some(z) => Ok(z),
        None => Err(MipsyError::Runtime(RuntimeError::new(Error::IntegerOverflow))),
    }
}

fn checked_sub(x: i32, y: i32) -> MipsyResult<i32> {
    match x.checked_sub(y) {
        Some(z) => Ok(z),
        None => Err(MipsyError::Runtime(RuntimeError::new(Error::IntegerOverflow))),
    }
}
