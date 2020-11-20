use mipsy_lib::State;
use mipsy_lib::Register;
use crate::InstSet;
use colored::*;
use mipsy_lib::{Binary, Runtime, RuntimeError, Uninitialised, Safe, decompile};

use crate::interactive::prompt;

fn get_last_mod<T, F>(runtime: &Runtime, f: F) -> Option<(usize, &State)>
where
    F: Fn(&State) -> T,
    T: PartialEq,
{
    let state = runtime.state();
    let initial_val = f(state);

    let mut i = runtime.timeline_len() - 2;
    loop {
        let old_state = runtime.nth_state(i).unwrap();

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

pub fn handle(
    error: RuntimeError,
    program: &str,
    iset: &InstSet,
    binary: &Binary,
    runtime: &Runtime,
) {
    println!();

    match error {
        RuntimeError::NoInstruction(addr) => {
            prompt::error(format!("couldn't find instruction at {}{:08x}", "0x".yellow(), addr));

            if let Some(prev_state) = runtime.prev_state() {
                if prev_state.get_pc() + 4 == addr {
                    prompt::tip_nl(format!("did you forget to `{} {}{}`?", "jr".bold(), "$".yellow(), "ra".bold()));
                } else if let Ok(inst) = prev_state.get_word(prev_state.get_pc()) {
                    prompt::tip(format!("try using `{}` - the instruction that brought you here was: ", "back".bold()));

                    crate::interactive::commands::util::print_inst(iset, binary, inst, prev_state.get_pc(), Some(program));
                    println!();
                }
            }
        }
        RuntimeError::Uninitialised(value) => {
            let (name, last_mod) = match value {
                Uninitialised::Byte(addr) | Uninitialised::Half(addr) | Uninitialised::Word(addr) => {
                    prompt::error_nl(format!("{}{:08x} is uninitialised", "0x".yellow(), addr));
                    return;
                }
                Uninitialised::Register(reg_num) => {
                    let name = Register::from_u32(reg_num).unwrap().to_lower_str();
                    let last_mod = get_last_mod(
                        runtime, 
                        |state| (state.is_register_written(reg_num), state.get_reg(reg_num).map(|t| Safe::valid(t)).unwrap_or(Safe::Uninitialised))
                    );

                    (name, last_mod)
                }
                Uninitialised::Lo => {
                    let name = "lo".into();
                    let last_mod = get_last_mod(
                        runtime, 
                        |state| (state.is_lo_written(), state.get_lo().map(|t| Safe::valid(t)).unwrap_or(Safe::Uninitialised))
                    );

                    (name, last_mod)
                }
                Uninitialised::Hi => {
                    let name = "hi".into();
                    let last_mod = get_last_mod(
                        runtime, 
                        |state| (state.is_hi_written(), state.get_hi().map(|t| Safe::valid(t)).unwrap_or(Safe::Uninitialised))
                    );

                    (name, last_mod)
                }
            };

            prompt::error(format!("your program tried to read uninitialised memory"));

            let state = runtime.state();
            let inst  = state.get_word(state.get_pc()).unwrap();
            let decompiled = decompile::decompile_inst_into_parts(binary, iset, inst, state.get_pc());

            eprintln!("\nthe instruction that failed was: ");
            crate::interactive::commands::util::print_inst_parts(binary, &decompiled, Some(program), false);
            eprintln!();

            eprintln!("this happened because {}{} was uninitialised", "$".yellow(), name.bold());
            if let Some((last_index, last_mod)) = last_mod {
                eprintln!("| the instruction that caused {}{} to become uninitialised was: ", "$".yellow(), name.bold());
                eprint!("| ");

                let last_inst = last_mod.get_word(last_mod.get_pc()).unwrap();
                crate::interactive::commands::util::print_inst(iset, binary, last_inst, last_mod.get_pc() - 4, Some(program));
            
                let distance = runtime.timeline_len() - last_index - 1;
                eprintln!("| to get back there, use `{} {}`", "back".bold(), distance.to_string().bold())
            } else {
                eprintln!("| note: {}{} was {} initialised", "$".yellow(), name.bold(), "never".bold());
            }

            println!();

        }
        RuntimeError::IntegerOverflow => {
            prompt::error("integer overflow");
            
            let state = runtime.state();
            let inst  = state.get_word(state.get_pc()).unwrap();
            let decompiled = decompile::decompile_inst_into_parts(binary, iset, inst, state.get_pc());

            eprintln!("\nthe instruction that failed was: ");
            crate::interactive::commands::util::print_inst_parts(binary, &decompiled, Some(program), false);
            eprintln!();

            let rs = (inst >> 21) & 0x1F;
            let rs_value = runtime.state().get_reg(rs).unwrap();
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
                    let value = runtime.state().get_reg(rt).unwrap();

                    eprintln!(
                        " - {}{} = {}",
                        "$".yellow(),
                        Register::from_u32(rt).unwrap().to_lower_str().bold(),
                        value
                    );

                    value
                };
            
            let adding = decompiled.inst_name.as_ref().map(|name| name.contains("add")).unwrap_or(false);

            let symbol   = if adding { "+" } else { "-" };
            let expected = if adding { rs_value.wrapping_add(value) } else { rs_value.wrapping_sub(value) };

            eprintln!("this happened because `{}` {} `{}` overflows past the limits of a 32-bit number\n", rs_value, symbol, value);
            prompt::tip(format!("if you expected the result to be {} (i.e. ignore overflow),", expected));

            if let Some(name) = match decompiled.inst_name.as_deref() {
                Some("add")  => Some("addu"),
                Some("addi") => Some("addiu"),
                Some("sub")  => Some("subu"),
                _ => None
            } {
                println!("     then try using the equivalent unsigned instruction: `{}`", name.bold());
            }

        }
        RuntimeError::DivisionByZero => {
            prompt::error("division by zero");

            let state = runtime.state();
            let inst  = state.get_word(state.get_pc()).unwrap();
            let decompiled = decompile::decompile_inst_into_parts(binary, iset, inst, state.get_pc());

            eprintln!("\nthe instruction that failed was: ");
            crate::interactive::commands::util::print_inst_parts(binary, &decompiled, Some(program), false);

            let rs = (inst >> 21) & 0x1F;
            let rt = (inst >> 16) & 0x1F;

            eprintln!("\nvalues:");
            eprintln!(
                " - {}{} = {}",
                "$".yellow(),
                Register::from_u32(rs).unwrap().to_lower_str().bold(),
                runtime.state().get_reg(rs).unwrap()
            );
            eprintln!(
                " - {}{} = {}",
                "$".yellow(),
                Register::from_u32(rt).unwrap().to_lower_str().bold(),
                runtime.state().get_reg(rt).unwrap()
            );

        }
        RuntimeError::SbrkNegative => {
            todo!()
        }
    }

    ()
}