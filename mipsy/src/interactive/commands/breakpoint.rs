use crate::interactive::{error::CommandError, prompt};

use super::*;
use colored::*;
use mipsy_parser::*;

pub(crate) fn breakpoint_command() -> Command {
    command(
        "breakpoint",
        vec!["br", "brk", "break"],
        vec!["i|d", "addr"],
        vec![],
        &format!(
            "{}nserts or {}eletes a breakpoint at an {}ess",
            "<i>".magenta(),
            "<d>".magenta(),
            "<addr>".magenta()
        ),
        &format!(
            "{0}nserts or {1}eletes a breakpoint at the specified {2}.\n\
             {2} may be: a decimal address (`4194304`), a hex address (`{3}400000`), or a label (`{4}`).\n\
             {5} must be `i`, `in`, `ins`, `insert`, or `add` to insert the breakpoint, or\n\
        \x20             `d`, `del`, `delete`, or `remove` to remove the breakpoint.\n\
             When running or stepping through your program, a breakpoint will cause execution to\n\
         \x20 pause temporarily, allowing you to debug the current state.\n\
             May error if provided a label that doesn't exist.\n\
           \n{6} you can also use the `{7}` MIPS instruction in your program's code!",
             "<i>".magenta(),
             "<d>".magenta(),
             "<address>".magenta(),
             "0x".yellow(),
             "main".yellow().bold(),
             "<i|d>".magenta(),
             format!(
                 "{}{}",
                 "tip".yellow().bold(), ":".bold(),
             ),
             "break".bold(),
        ),
        |state, _label, args| {
            let remove = match &*args[0] {
                "i" | "in" | "ins" | "insert" => false,
                "d" | "del" | "delete" | "remove" => true,
                _ => return Err(
                    CommandError::WithTip {
                        error: Box::new(CommandError::BadArgument {
                            arg: "<i|d>".magenta().to_string(),
                            instead: args[0].to_string()
                        }),
                        tip: format!("try `{}`", "help breakpoint".bold())
                }),
            };

            let get_error = || CommandError::WithTip { 
                error: Box::new(CommandError::BadArgument { arg: "<addr>".magenta().to_string(), instead: args[0].to_string() }),
                tip: format!("try `{}`", "help breakpoint".bold()),
            };

            let arg = mipsy_parser::parse_argument(&args[1])
                    .map_err(|_| get_error())?;

            let binary = state.binary.as_mut().ok_or_else(|| CommandError::MustLoadFile)?;

            match arg {
                MPArgument::Number(MPNumber::Immediate(ref imm)) => {
                    let (addr, is_label) = match imm {
                        MPImmediate::I16(imm) => {
                            (*imm as u32, false)
                        }
                        MPImmediate::U16(imm) => {
                            (*imm as u32, false)
                        }
                        MPImmediate::I32(imm) => {
                            (*imm as u32, false)
                        }
                        MPImmediate::U32(imm) => {
                            (*imm, false)
                        }
                        MPImmediate::LabelReference(label) => {
                            (
                                binary.get_label(&label)
                                    .map_err(|_| CommandError::UnknownLabel { label: label.to_string() })?,
                                true
                            )
                        }
                    };

                    if addr % 4 != 0 {
                        prompt::error_nl(format!("address 0x{:08x} should be word-aligned", addr));
                        return Ok(());
                    }

                    if remove {
                        if !binary.breakpoints.contains(&addr) {
                            prompt::error_nl(format!(
                                "breakpoint at {} doesn't exist", 
                                if is_label {
                                    args[1].yellow().bold().to_string()
                                } else {
                                    args[1].to_string()
                                }
                            ));

                            return Ok(());
                        }
                    } else {
                        if binary.breakpoints.contains(&addr) {
                            prompt::error_nl(format!(
                                "breakpoint at {} already exists", 
                                if is_label {
                                    args[1].yellow().bold().to_string()
                                } else {
                                    args[1].to_string()
                                }
                            ));

                            return Ok(());
                        }
                    }

                    let action = 
                        if remove {
                            binary.breakpoints.retain(|&x| addr != x);
                            "removed"
                        } else {
                            binary.breakpoints.push(addr);
                            "inserted"
                        };

                    if is_label {
                        prompt::success_nl(format!("breakpoint {} at {} (0x{:08x})", action, args[1].yellow().bold(), addr));
                    } else {
                        prompt::success_nl(format!("breakpoint {} at 0x{:08x}", action, addr));
                    }
                }
                _ => return Err(get_error()),
            }

            Ok(())
        }
    )
}
