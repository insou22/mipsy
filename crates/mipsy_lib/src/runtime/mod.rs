mod unsafe_cow;
mod state;

use std::collections::HashMap;

use crate::{Binary, DATA_BOT, HEAP_BOT, KDATA_BOT, KTEXT_BOT, MipsyError, MipsyResult, Register, RuntimeError, STACK_TOP, Safe, TEXT_BOT, Uninitialised, error::runtime::Error, runtime::state::State};

use self::state::Timeline;

pub const NUL:  u8  = 0;
pub const NULL: u32 = 0;
pub const PAGE_SIZE: u32 = 64;

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

pub struct Runtime {
    timeline: Timeline,
}

impl Runtime {
    pub fn step(mut self) -> Result<SteppedRuntime, (Runtime, MipsyError)> {
        let state = self.timeline.push_next_state();

        let inst = state.read_mem_word(state.pc())
                .map_err(|_| (self, MipsyError::Runtime(RuntimeError::new(Error::UnknownInstruction { addr: state.pc() }))))?;

        state.set_pc(state.pc() + 4);

        match self.execute(inst) {
            Err(err) => {
                self.timeline.pop_last_state();

                Err(err)
            }
            ok => ok,
        }
    }

    fn execute(mut self, inst: u32) -> Result<SteppedRuntime, (Runtime, MipsyError)>
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
                self.execute_r(funct, rd, rs, rt, shamt)
            }
            0b000010 | 0b000011 => {
                // J-Type
                self.execute_j(opcode, addr);

                Ok(Ok(self))
            }
            _ => {
                // I-Type
                self.execute_i(opcode, rs, rt, imm)
                    .map_err(|err| (self, err))?;

                Ok(Ok(self))
            }
        }
    }

    // remove when floating point syscalls are finished
    #[allow(unreachable_code)]
    fn syscall(self) -> Result<RuntimeSyscallGuard, (Runtime, MipsyError)> {
        let state = self.timeline.state();

        (|| {
            let syscall = state.read_register(Register::V0.to_u32())?;
        
            Ok(
                match syscall {
                    SYS1_PRINT_INT => RuntimeSyscallGuard::PrintInt(
                        PrintIntArgs {
                            value: state.read_register(Register::A0.to_u32())?
                        },
                        self
                    ),
                    SYS2_PRINT_FLOAT => RuntimeSyscallGuard::PrintFloat(
                        PrintFloatArgs {
                            value: todo!(),
                        },
                        self
                    ),
                    SYS3_PRINT_DOUBLE => RuntimeSyscallGuard::PrintDouble(
                        PrintDoubleArgs {
                            value: todo!(),
                        },
                        self
                    ),
                    SYS4_PRINT_STRING => RuntimeSyscallGuard::PrintString(
                        PrintStringArgs {
                            value: state.read_mem_string(
                                state.read_register(Register::A0.to_u32())? as _
                            )?,
                        },
                        self
                    ),
                    SYS5_READ_INT => RuntimeSyscallGuard::ReadInt(
                        Box::new(move |value| {
                            state.write_register(Register::V0.to_u32(), value);
                            self
                        })
                    ),
                    SYS6_READ_FLOAT => RuntimeSyscallGuard::ReadFloat(
                        todo!()
                    ),
                    SYS7_READ_DOUBLE => RuntimeSyscallGuard::ReadDouble(
                        todo!()
                    ),
                    SYS8_READ_STRING => {
                        let buf = state.read_register(Register::A0.to_u32())? as u32;
                        let len = state.read_register(Register::A1.to_u32())? as _;

                        RuntimeSyscallGuard::ReadString(
                            ReadStringArgs {
                                max_len: len,
                            },
                            Box::new(move |mut string| {
                                if string.len() >= len as usize {
                                    string.resize(len.max(0) as _, 0);
                                }
                                
                                for (i, byte) in string.into_iter().enumerate() {
                                    state.write_mem_byte(buf + i as u32, byte);
                                }

                                self
                            })
                        )
                    }
                    SYS9_SBRK => {
                        let bytes = state.read_register(Register::A0.to_u32())?;

                        if bytes > 0 {
                            state.set_heap_size(state.heap_size().saturating_add(bytes as _));
                        } else if bytes < 0 {
                            state.set_heap_size(state.heap_size().saturating_sub(bytes.abs() as _));
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
                            value: state.read_register(Register::A0.to_u32())? as _,
                        },
                        self
                    ),
                    SYS12_READ_CHAR => RuntimeSyscallGuard::ReadChar(
                        Box::new(move |value| {
                            state.write_register(Register::V0.to_u32(), value as _);
                            self
                        })
                    ),
                    SYS13_OPEN => RuntimeSyscallGuard::Open(
                        OpenArgs {
                            path: state.read_mem_string(
                                state.read_register(Register::A0.to_u32())? as _
                            )?,
                            flags: state.read_register(Register::A1.to_u32())? as _,
                            mode:  state.read_register(Register::A2.to_u32())? as _,
                        },
                        Box::new(move |fd| {
                            state.write_register(Register::V0.to_u32(), fd as _);
                            self
                        })
                    ),
                    SYS14_READ => {
                        let fd  = state.read_register(Register::A0.to_u32())? as _;
                        let buf = state.read_register(Register::A1.to_u32())? as u32;
                        let len = state.read_register(Register::A2.to_u32())? as _;

                        RuntimeSyscallGuard::Read(
                            ReadArgs {
                                fd,
                                len,
                            },
                            Box::new(move |bytes| {
                                let len = (len as usize).min(bytes.len());

                                bytes[..len].iter().enumerate().for_each(|(i, byte)| {
                                    state.write_mem_byte(buf + i as u32, *byte);
                                });
                                state.write_register(Register::V0.to_u32(), len as _);
                                
                                self
                            })
                        )
                    }
                    SYS15_WRITE => {
                        let fd  = state.read_register(Register::A0.to_u32())? as _;
                        let buf = state.read_register(Register::A1.to_u32())? as _;
                        let len = state.read_register(Register::A2.to_u32())? as _;

                        RuntimeSyscallGuard::Write(
                            WriteArgs {
                                fd,
                                buf: state.read_mem_bytes(buf, len)?,
                            },
                            Box::new(move |written| {
                                state.write_register(Register::V0.to_u32(), written as _);
                                
                                self
                            })
                        )
                    }
                    SYS16_CLOSE => RuntimeSyscallGuard::Close(
                        CloseArgs {
                            fd: state.read_register(Register::A0.to_u32())? as _,
                        },
                        Box::new(move |status| {
                            state.write_register(Register::V0.to_u32(), status as _);
                            self
                        })
                    ),
                    SYS17_EXIT_STATUS => RuntimeSyscallGuard::ExitStatus(
                        ExitStatusArgs {
                            exit_code: state.read_register(Register::A0.to_u32())? as _,
                        },
                        self
                    ),
                }
            )
        })().map_err(|err| (self, err))
    }

    fn execute_r(mut self, funct: u32, rd: u32, rs: u32, rt: u32, shamt: u32) -> Result<SteppedRuntime, (Runtime, MipsyError)>
    {
        let state = self.timeline.state_mut();

        match funct {
            // SYSCALL
            0x0C => { Ok(Err(self.syscall()?)) },

            // BREAK
            0x0D => { Ok(Err(RuntimeSyscallGuard::Breakpoint(self))) },

            _ => {
                (|| {
                    match funct {
                        // SLL  $Rd, $Rt, Sa
                        0x00 => { state.write_register(rd, (state.read_register(rt)? << shamt) as i32); },
            
                        // Unused
                        0x01 => {},
            
                        // SRL  $Rd, $Rt, Sa
                        0x02 => { state.write_register(rd, (state.read_register(rt)? >> shamt) as i32); },
            
                        // SRA  $Rd, $Rt, Sa
                        0x03 => { state.write_register(rd, state.read_register(rt)? >> shamt); },
            
                        // SLLV $Rd, $Rt, $Rs
                        0x04 => { state.write_register(rd, (state.read_register(rt)? << state.read_register(rs)?) as i32); },
            
                        // Unused
                        0x05 => {},
            
                        // SRLV $Rd, $Rt, $Rs
                        0x06 => { state.write_register(rd, (state.read_register(rt)? >> state.read_register(rs)?) as i32); },
            
                        // SRAV $Rd, $Rt, $Rs
                        0x07 => { state.write_register(rd, state.read_register(rt)? >> state.read_register(rs)?); },
            
                        // JR   $Rs
                        0x08 => { state.set_pc(state.read_register(rs)? as u32); },
            
                        // JALR $Rs
                        0x09 => { 
                            state.write_register(Register::Ra.to_number() as u32, state.pc() as _); 
                            state.set_pc(state.read_register(rs)? as _);
                        },
                        
                        // Unused
                        0x0A => {},
            
                        // Unused
                        0x0B => {},
            
                        // SYSCALL
                        0x0C => unreachable!("covered above"),
            
                        // BREAK
                        0x0D => unreachable!("covered above"),
            
                        // Unused
                        0x0E => {},
            
                        // Unused
                        0x0F => {},
            
                        // MFHI $Rd
                        0x10 => { state.write_register(rd, state.read_hi()?); },
            
                        // MTHI $Rs
                        0x11 => { state.write_hi(state.read_register(rs)?); },
            
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
                            let rs_val = state.read_register(rs)?;
                            let rt_val = state.read_register(rt)?;
            
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
                            let rs_val = state.read_register(rs)?;
                            let rt_val = state.read_register(rt)?;
            
                            if rt_val == 0 {
                                return Err(MipsyError::Runtime(RuntimeError::new(Error::DivisionByZero)));
                            }
            
                            state.write_lo(rs_val / rt_val);
                            state.write_hi(rs_val % rt_val);
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

                    Ok(Ok(self))
                })().map_err(|err| (self, err))
            }
        }
    }

    fn execute_i(&mut self, opcode: u32, rs: u32, rt: u32, imm: i16) -> MipsyResult<()> {
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
            0x20 => { state.write_register(rt, state.read_mem_byte(state.read_register(rs)?.wrapping_add(imm_sign_extend) as _)? as i8 as _); },
            
            // LH   $Rt, Im($Rs)
            0x21 => { state.write_register(rt, state.read_mem_half(state.read_register(rs)?.wrapping_add(imm_sign_extend) as _)? as i16 as _); },
            
            // Unused
            0x22 => {},
            
            // LW   $Rt, Im($Rs)
            0x23 => { state.write_register(rt, state.read_mem_word(state.read_register(rs)?.wrapping_add(imm_sign_extend) as _)? as _); },
            
            // LBU  $Rt, Im($Rs)
            0x24 => { state.write_register(rt, state.read_mem_byte(state.read_register(rs)?.wrapping_add(imm_sign_extend) as _)? as _); },
            
            // LHU  $Rt, Im($Rs)
            0x25 => { state.write_register(rt, state.read_mem_half(state.read_register(rs)?.wrapping_add(imm_sign_extend) as _)? as _); },
            
            // Unused
            0x26 => {},
            
            // Unused
            0x27 => {},
            
            // SB   $Rt, Im($Rs)
            0x28 => { state.write_mem_byte(rt, state.read_register(rs)?.wrapping_add(imm_sign_extend) as _); },
            
            // SH   $Rt, Im($Rs)
            0x29 => { state.write_mem_half(rt, state.read_register(rs)?.wrapping_add(imm_sign_extend) as _); },
            
            // Unused
            0x2A => {},
            
            // SW   $Rt, Im($Rs)
            0x2B => { state.write_mem_word(rt, state.read_register(rs)?.wrapping_add(imm_sign_extend) as _); },
            
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
    ReadChar   (           Box<dyn FnOnce(u8)      -> Runtime>),
    Open       (OpenArgs,  Box<dyn FnOnce(u32)     -> Runtime>),
    Read       (ReadArgs,  Box<dyn FnOnce(Vec<u8>) -> Runtime>),
    Write      (WriteArgs, Box<dyn FnOnce(u32)     -> Runtime>),
    Close      (CloseArgs, Box<dyn FnOnce(i32)     -> Runtime>),
    ExitStatus (ExitStatusArgs, Runtime),

    // other
    Breakpoint (Runtime),
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
    path: Vec<u8>,
    flags: u32,
    mode: u32,
}

pub struct ReadArgs {
    fd: u32,
    len: u32,
}

pub struct WriteArgs {
    fd: u32,
    buf: Vec<u8>,
}

pub struct CloseArgs {
    fd: u32,
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
    fn extend_sign(self) -> i32;
}

impl ExtendSign for u8 {
    fn extend_sign(self) -> i32 {
        self as i8 as i32
    }
}

impl ExtendSign for u16 {
    fn extend_sign(self) -> i32 {
        self as i16 as i32
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
                hi: Default::default(),
                lo: Default::default(),
            };

        let mut text_addr = TEXT_BOT;
        for &word in &program.text {
            initial_state.write_mem_word(text_addr, word);
            text_addr += 4;
        }

        let mut data_addr = DATA_BOT;
        for &byte in &program.data {
            match byte {
                Safe::Valid(byte) => initial_state.write_mem_byte(data_addr, byte),
                Safe::Uninitialised => {}
            }

            data_addr += 1;
        }

        let mut ktext_addr = KTEXT_BOT;
        for &word in &program.ktext {
            initial_state.write_mem_word(ktext_addr, word);
            ktext_addr += 4;
        }

        let mut kdata_addr = KDATA_BOT;
        for &byte in &program.kdata {
            match byte {
                Safe::Valid(byte) => initial_state.write_mem_byte(kdata_addr, byte),
                Safe::Uninitialised => {}
            }

            kdata_addr += 1;
        }

        initial_state.write_register(Register::Zero.to_number() as _, 0);
        initial_state.write_register(Register::Sp.to_number()   as _, STACK_TOP as _);
        initial_state.write_register(Register::Fp.to_number()   as _, STACK_TOP as _);
        initial_state.write_register(Register::Gp.to_number()   as _, HEAP_BOT  as _);

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
        let strings_stack_addr = STACK_TOP - total_strings_len;

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
                state.write_mem_word(star_addr, string_addr);

                for byte in arg.bytes() {
                    state.write_mem_byte(string_addr, byte);

                    string_addr += 1;
                }

                // null terminator
                state.write_mem_byte(string_addr, NUL);
                string_addr += 1;

                star_addr += 4;
            }

            state.write_mem_word(star_addr, NULL);
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