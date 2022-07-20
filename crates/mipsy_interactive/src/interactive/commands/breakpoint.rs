use crate::interactive::{Breakpoint, error::CommandError, prompt};

use super::*;
use colored::*;
use mipsy_parser::*;

pub(crate) fn breakpoint_command() -> Command {
    command(
        "breakpoint",
        vec!["br", "brk", "break"],
        vec!["addr"],
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
           \n{6}{7} you can also use the `{8}` MIPS instruction in your program's code!",
             "<i>".magenta(),
             "<d>".magenta(),
             "<address>".magenta(),
             "0x".yellow(),
             "main".yellow().bold(),
             "<i|d>".magenta(),
             "tip".yellow().bold(),
             ":".bold(),
             "break".bold(),
        ),
        |state, _label, args| {
            return match &*args[0] {
                "list" => breakpoint_list(state, _label, &args[1..]),
                _      => breakpoint_insert(state, _label, &args),
            }
        }
    )
}

fn breakpoint_insert(state: &mut State, _label: &str, mut args: &[String]) -> Result<(), CommandError> {
    // todo(joshh): try to allow breakpoints to be inserted at labels sharing names with reserved
    // keywords
    
    let temporary = if &*args[0] == "temporary" {
        args = &args[1..];
        true
    } else {
        false
    };

    let get_error = || CommandError::WithTip { 
        // TODO(joshh): fix error msgs
        error: Box::new(CommandError::BadArgument { arg: "<addr>".magenta().to_string(), instead: args[0].to_string() }),
        tip: format!("try `{}`", "help breakpoint".bold()),
    };

    let arg = mipsy_parser::parse_argument(&args[0], state.config.tab_size)
            .map_err(|_| get_error())?;

    let binary = state.binary.as_mut().ok_or(CommandError::MustLoadFile)?;

    match arg {
        MpArgument::Number(MpNumber::Immediate(ref imm)) => {
            let (addr, is_label) = match imm {
                MpImmediate::I16(imm) => {
                    (*imm as u32, false)
                }
                MpImmediate::U16(imm) => {
                    (*imm as u32, false)
                }
                MpImmediate::I32(imm) => {
                    (*imm as u32, false)
                }
                MpImmediate::U32(imm) => {
                    (*imm, false)
                }
                MpImmediate::LabelReference(label) => {
                    (
                        binary.get_label(label)
                            .map_err(|_| CommandError::UnknownLabel { label: label.to_string() })?,
                        true
                    )
                }
            };

            if addr % 4 != 0 {
                prompt::error_nl(format!("address 0x{:08x} should be word-aligned", addr));
                return Ok(());
            }

            if state.breakpoints.contains_key(&addr) {
                prompt::error_nl(format!(
                    "breakpoint at {} already exists", 
                    if is_label {
                        args[0].yellow().bold().to_string()
                    } else {
                        args[0].to_string()
                    }
                ));

                return Ok(());
            }

            // TODO(joshh): kinda cringe
            let action = {
                let id = state.generate_breakpoint_id();
                state.breakpoints.insert(id, Breakpoint::new(addr));

                "inserted"
            };

            if is_label {
                prompt::success_nl(format!("breakpoint {} at {} (0x{:08x})", action, args[0].yellow().bold(), addr));
            } else {
                prompt::success_nl(format!("breakpoint {} at 0x{:08x}", action, addr));
            }
        }
        _ => return Err(get_error()),
    }
    Ok(())
}

fn breakpoint_list(state: &mut State, _label: &str, _args: &[String]) -> Result<(), CommandError> {
    let binary = state.binary.as_mut().ok_or(CommandError::MustLoadFile)?;

    if state.breakpoints.is_empty() {
        prompt::error_nl("no breakpoints set");
        return Ok(());
    }

    let mut breakpoints = state.breakpoints.values()
            .map(|bp| {
                (
                    bp.addr,
                    binary.labels.iter()
                        .find(|(_, &val)| val == bp.addr)
                        .map(|(name, _)| (
                            format!("{}", name.yellow().bold()),
                            name.len()
                        ))
                )
            })
            .collect::<Vec<(u32, Option<(String, usize)>)>>();

    breakpoints.sort_by_key(|(addr, _)| *addr);

    let max_len = breakpoints.iter()
            .map(|(_, lbl)| {
                lbl.as_ref()
                    .map(|(_, len)| *len)
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0);

    println!("\n{}", "[breakpoints]".green().bold());
    for (addr, text) in breakpoints {
        match text {
            Some((name, len)) => {
                println!("{}{} ({}{:08x})", name, " ".repeat(max_len - len), "0x".yellow(), addr);
            }
            None => {
                println!("{}  {}{:08x}", " ".repeat(max_len), "0x".yellow(), addr);
            }
        }
    }
    println!();

    Ok(())
}
