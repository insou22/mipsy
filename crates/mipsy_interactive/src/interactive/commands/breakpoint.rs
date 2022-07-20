use crate::interactive::{Breakpoint, error::CommandError, prompt};
use std::iter::successors;

use super::*;
use colored::*;
use mipsy_parser::*;

enum BpState {
    Enable,
    Disable,
    Toggle,
}

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
        |state, label, args| {
            // TODO(joshh): match on label for breakpoints aliases?
            return match &*args[0] {
                "l" | "list" =>
                    breakpoint_list  (state, label, &args[1..]),
                "del" | "delete" | "remove" =>
                    breakpoint_insert(state, label, &args[1..], true),
                "e" | "enable" =>
                    breakpoint_toggle(state, label, &args[1..], BpState::Enable),
                "d" | "disable" =>
                    breakpoint_toggle(state, label, &args[1..], BpState::Disable),
                "toggle" =>
                    breakpoint_toggle(state, label, &args[1..], BpState::Toggle),
                _ =>
                    breakpoint_insert(state, label, &args, false),
            }
        }
    )
}

fn breakpoint_insert(state: &mut State, _label: &str, mut args: &[String], remove: bool) -> Result<(), CommandError> {
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

            let id;
            let action = if remove {
                if let Some(br) = state.breakpoints.iter().find(|&i| i.1.addr == addr) {
                    id = *br.0;
                    state.breakpoints.remove(&id);
                    "removed"
                } else {
                    prompt::error_nl(format!(
                        "breakpoint at {} doesn't exist",
                        if is_label {
                            args[0].yellow().bold().to_string()
                        } else {
                            args[0].to_string()
                        }
                    ));
                    return Ok(());
                }
            } else {
                if !state.breakpoints.values().any(|br| br.addr == addr) {
                    id = state.generate_breakpoint_id();
                    state.breakpoints.insert(id, Breakpoint::new(addr));
                    "inserted"
                } else {
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
            };

            if is_label {
                prompt::success_nl(format!("breakpoint {} {} at {} (0x{:08x})", id, action, args[0].yellow().bold(), addr));
            } else {
                prompt::success_nl(format!("breakpoint {} {} at 0x{:08x}",      id, action, addr));
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

    let mut breakpoints = state.breakpoints.iter()
            .map(|x| {
                let (&id, bp) = x;

                (
                    (
                        id,
                        successors(Some(id), |&id| (id >= 10).then(|| id / 10)).count(),
                    ),
                    bp.addr,
                    binary.labels.iter()
                        .find(|(_, &val)| val == bp.addr)
                        .map(|(name, _)| (
                            format!("{}", name.yellow().bold()),
                            name.len()
                        ))
                )
            })
            .collect::<Vec<_>>();

    breakpoints.sort_by_key(|(_, addr, _)| *addr);

    let max_label_len = breakpoints.iter()
            .map(|(_, _, lbl)| {
                lbl.as_ref()
                    .map(|(_, len)| *len)
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0);

    let max_id_len = breakpoints.iter()
            .map(|&(id, _, _)| {
                id.1
            })
            .max()
            .unwrap_or(0);

    println!("\n{}", "[breakpoints]".green().bold());
    for (id, addr, text) in breakpoints {
        match text {
            Some((name, len)) => {
                println!("{}{}: {}{} ({}{:08x})", " ".repeat(max_id_len - id.1), id.0, name, " ".repeat(max_label_len - len), "0x".yellow(), addr);
            }
            None => {
                println!("{}{}: {}  {}{:08x}",    " ".repeat(max_id_len - id.1), id.0, " ".repeat(max_label_len), "0x".yellow(), addr);
            }
        }
    }
    println!();

    Ok(())
}

fn breakpoint_toggle(state: &mut State, _label: &str, mut args: &[String], enabled: BpState) -> Result<(), CommandError> {
    // TODO(joshh): reduce repetition
    let get_error = || CommandError::WithTip { 
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

            let id;
            if let Some(x) = state.breakpoints.iter().find(|&i| i.1.addr == addr) {
                id = *x.0;
                let mut br = state.breakpoints.get_mut(&id).unwrap();
                br.enabled = match enabled {
                    BpState::Enable  => true,
                    BpState::Disable => false,
                    BpState::Toggle  => !br.enabled,
                }
            } else {
                prompt::error_nl(format!(
                    "breakpoint at {} doesn't exist",
                    if is_label {
                        args[0].yellow().bold().to_string()
                    } else {
                        args[0].to_string()
                    }
                ));
                return Ok(());
            }

            // already ruled out possibility of entry not existing
            let action = match state.breakpoints.get(&id).unwrap().enabled {
                true  => "enabled",
                false => "disabled",
            };

            if is_label {
                prompt::success_nl(format!("breakpoint {} {} at {} (0x{:08x})", id, action, args[0].yellow().bold(), addr));
            } else {
                prompt::success_nl(format!("breakpoint {} {} at 0x{:08x}",      id, action, addr));
            }
        }
        _ => return Err(get_error()),
    }
    Ok(())
}
