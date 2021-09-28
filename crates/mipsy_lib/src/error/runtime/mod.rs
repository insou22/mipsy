use std::rc::Rc;

use colored::Colorize;
use crate::{Binary, InstSet, Register, Runtime, Safe, State, decompile, runtime::state::{WRITE_MARKER_HI, WRITE_MARKER_LO}};

use super::util::{inst_parts_to_string, inst_to_string, tip_header};

#[derive(Debug)]
pub struct RuntimeError {
    error: Error,
}

impl RuntimeError {
    pub fn new(error: Error) -> Self {
        Self {
            error,
        }
    }

    pub fn error(&self) -> &Error {
        &self.error
    }

    pub fn show_error(&self, context: ErrorContext, source_code: Vec<(Rc<str>, Rc<str>)>, inst_set: &InstSet, binary: &Binary, runtime: &Runtime) {
        print!("{}{} ", "error".bright_red().bold(), ":".bold());
        println!("{}", self.error.message(context, source_code, inst_set, binary, runtime));

        for tip in self.error.tips(inst_set, binary, runtime) {
            println!("{} {}", tip_header(), tip);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ErrorContext {
    Binary,
    Interactive,
    Repl,
}

#[derive(Debug)]
pub enum Error {
    UnknownInstruction { addr: u32 },
    Uninitialised { value: Uninitialised },

    IntegerOverflow,
    DivisionByZero,
    SbrkNegative,
}

#[derive(Debug)]
pub enum Uninitialised {
    Byte { addr: u32 },
    Half { addr: u32 },
    Word { addr: u32 },
    Register { reg_num: u32 },
    Lo,
    Hi,
}

impl Error {
    pub fn message(&self, context: ErrorContext, source_code: Vec<(Rc<str>, Rc<str>)>, inst_set: &InstSet, binary: &Binary, runtime: &Runtime) -> String {
        match self {
            Error::UnknownInstruction { addr } => {
                let message = "could not find instruction at";
                let zero_x = "0x".yellow();

                format!("{} {}{:08x}\n", message, zero_x, addr)
            }

            Error::Uninitialised { value } => {
                let (name, last_mod) = match value {
                    Uninitialised::Byte { addr } | Uninitialised::Half { addr } | Uninitialised::Word { addr } => {
                        let message = "is uninitialised";
                        let zero_x = "0x".yellow();
                        return format!("{}{:08x} {}", zero_x, addr, message);
                    }

                    Uninitialised::Register { reg_num } => {
                        let name = Register::from_u32(*reg_num).unwrap().to_lower_str();
                        let last_mod = get_last_mod(
                            runtime, 
                            |state| {
                                (
                                    (state.write_marker() & 1u64 << *reg_num) != 0,

                                    state.read_register(*reg_num)
                                        .map(Safe::valid)
                                        .unwrap_or(Safe::Uninitialised)
                                )
                            }
                        );
    
                        (name, last_mod)
                    }
                    
                    Uninitialised::Lo => {
                        let name = "lo";
                        let last_mod = get_last_mod(
                            runtime, 
                            |state| {
                                (
                                    (state.write_marker() & 1u64 << WRITE_MARKER_LO) != 0,
                                    
                                    state.read_lo()
                                        .map(Safe::valid)
                                        .unwrap_or(Safe::Uninitialised)
                                )
                            }
                        );
    
                        (name, last_mod)
                    }
                    
                    Uninitialised::Hi => {
                        let name = "hi";
                        let last_mod = get_last_mod(
                            runtime, 
                            |state| {
                                (
                                    (state.write_marker() & 1u64 << WRITE_MARKER_HI) != 0,
                                    
                                    state.read_hi()
                                        .map(Safe::valid)
                                        .unwrap_or(Safe::Uninitialised)
                                )
                            }
                        );
    
                        (name, last_mod)
                    }
                };

                let mut error = String::new();
                error.push_str("your program tried to read uninitialised memory\n");

                let state = runtime.timeline().state();
                let inst  = state.read_mem_word(state.pc()).unwrap();
                let decompiled = decompile::decompile_inst_into_parts(binary, inst_set, inst, state.pc());
    
                if let ErrorContext::Binary | ErrorContext::Interactive = context {
                    error.push_str("the instruction that failed was:\n");
                    error.push_str(&inst_parts_to_string(&decompiled, &source_code, binary, false));
                    error.push('\n');
                }

                error.push_str(&format!("this happened because {}{} was uninitialised\n", "$".yellow(), name.bold()));

                if let Some((last_index, last_mod)) = last_mod {
                    error.push_str(&format!("{} the instruction that caused {}{} to become uninitialised was:\n", "|".red(),  "$".yellow(), name.bold()));
                    error.push_str(&"|".red());
                    error.push(' ');
    
                    let last_inst = last_mod.read_mem_word(last_mod.pc()).unwrap();

                    error.push_str(&inst_to_string(last_inst, last_mod.pc() - 4, &source_code, binary, inst_set, false));
                    error.push('\n');

                    if let ErrorContext::Interactive = context {
                        let distance = runtime.timeline().timeline_len() - last_index - 1;
                        error.push_str(&format!("{}\n{0} to get back there, use `{} {}`\n", "|".red(), "back".bold(), distance.to_string().bold()));
                    }
                } else {(
                    error.push_str(&format!("{} note: {}{} was {} initialised\n", "|".red(), "$".yellow(), name.bold(), "never".bold())));
                }
    
                error.push('\n');

                error
            }

            Error::IntegerOverflow => {
                let mut error = String::new();
                error.push_str("integer overflow\n");
                
                let state = runtime.timeline().state();
                let inst  = state.read_mem_word(state.pc()).unwrap();
                let decompiled = decompile::decompile_inst_into_parts(binary, inst_set, inst, state.pc());

                if let ErrorContext::Binary | ErrorContext::Interactive = context {
                    error.push_str("\nthe instruction that failed was:\n");
                    error.push_str(&inst_parts_to_string(&decompiled, &source_code, binary, false));
                    error.push('\n');
                }

                let rs = (inst >> 21) & 0x1F;
                let rs_value = runtime.timeline().state().read_register(rs).unwrap();
                error.push_str("values:\n");
                error.push_str(&format!(
                    " - {}{} = {}\n",
                    "$".yellow(),
                    Register::from_u32(rs).unwrap().to_lower_str().bold(),
                    rs_value,
                ));

                let value = 
                    if let Ok(imm) = decompiled.arguments[2].parse::<i16>() {
                        imm as i32
                    } else {
                        let rt = (inst >> 16) & 0x1F;
                        let value = runtime.timeline().state().read_register(rt).unwrap();

                        error.push_str(&format!(
                            " - {}{} = {}\n",
                            "$".yellow(),
                            Register::from_u32(rt).unwrap().to_lower_str().bold(),
                            value
                        ));

                        value
                    };
                
                let adding = decompiled.inst_name.as_ref().map(|name| name.contains("add")).unwrap_or(false);

                let symbol   = if adding { "+" } else { "-" };

                error.push_str(&format!("this happened because `{}` {} `{}` overflows past the limits of a 32-bit number\n\n", rs_value, symbol, value));

                error
            }
            
            Error::DivisionByZero => {
                let mut error = String::new();

                error.push_str("division by zero\n");

                let state = runtime.timeline().state();
                let inst  = state.read_mem_word(state.pc()).unwrap();
                let decompiled = decompile::decompile_inst_into_parts(binary, inst_set, inst, state.pc());

                if let ErrorContext::Binary | ErrorContext::Interactive = context {
                    error.push_str("\nthe instruction that failed was:\n");
                    error.push_str(&inst_parts_to_string(&decompiled, &source_code, binary, false));
                    error.push('\n');
                }

                let rs = (inst >> 21) & 0x1F;
                let rt = (inst >> 16) & 0x1F;

                error.push_str("\nvalues:\n");

                error.push_str(&format!(
                    " - {}{} = {}\n",
                    "$".yellow(),
                    Register::from_u32(rs).unwrap().to_lower_str().bold(),
                    runtime.timeline().state().read_register(rs).unwrap()
                ));

                error.push_str(&format!(
                    " - {}{} = {}\n",
                    "$".yellow(),
                    Register::from_u32(rt).unwrap().to_lower_str().bold(),
                    runtime.timeline().state().read_register(rt).unwrap()
                ));

                error
            }
            
            Error::SbrkNegative => {
                todo!()
            }
        }
    }

    pub fn tips(&self, inst_set: &InstSet, binary: &Binary, runtime: &Runtime) -> Vec<String> {
        match self {
            Error::UnknownInstruction { .. } => {
                vec![]
            }
            Error::Uninitialised { .. } => {
                vec![]
            }
            Error::IntegerOverflow => {
                let mut tip = String::new();

                let state = runtime.timeline().state();
                let inst  = state.read_mem_word(state.pc()).unwrap();
                let decompiled = decompile::decompile_inst_into_parts(binary, inst_set, inst, state.pc());

                let adding = decompiled.inst_name.as_ref().map(|name| name.contains("add")).unwrap_or(false);

                let rs = (inst >> 21) & 0x1F;
                let rs_value = runtime.timeline().state().read_register(rs).unwrap();
                eprintln!("values:");
                eprintln!(
                    " - {}{} = {}",
                    "$".yellow(),
                    Register::from_u32(rs).unwrap().to_lower_str().bold(),
                    rs_value,
                );
                
                let value = 
                    if let Ok(imm) = decompiled.arguments[2].parse::<i16>() {
                        imm as i32
                    } else {
                        let rt = (inst >> 16) & 0x1F;
                        let value = runtime.timeline().state().read_register(rt).unwrap();
    
                        eprintln!(
                            " - {}{} = {}",
                            "$".yellow(),
                            Register::from_u32(rt).unwrap().to_lower_str().bold(),
                            value
                        );
    
                        value
                    };

                let expected = if adding { rs_value.wrapping_add(value) } else { rs_value.wrapping_sub(value) };
    

                tip.push_str(&format!("if you expected the result to be {} (i.e. ignore overflow),\n", expected));

                if let Some(name) = match decompiled.inst_name.as_deref() {
                    Some("add")  => Some("addu"),
                    Some("addi") => Some("addiu"),
                    Some("sub")  => Some("subu"),
                    _ => unreachable!()
                } {
                    tip.push_str(&format!("     then try using the equivalent unsigned instruction: `{}`\n", name.bold()));
                }
    
                vec![tip]
            }
            Error::DivisionByZero => {
                vec![]
            }
            Error::SbrkNegative => {
                vec![]
            }
        }
    }
}

fn get_last_mod<T, F>(runtime: &Runtime, f: F) -> Option<(usize, &State)>
where
    F: Fn(&State) -> T,
    T: PartialEq,
{
    let state = runtime.timeline().state();
    let initial_val = f(state);

    let mut i = runtime.timeline().timeline_len() - 2;
    loop {
        let old_state = runtime.timeline().nth_state(i).unwrap();

        if initial_val != f(old_state) {
            return Some((i, old_state));
        }

        if i == 0 {
            break;
        }

        i -= 1;
    }

    None
}
