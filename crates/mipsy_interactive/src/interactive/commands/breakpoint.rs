use crate::interactive::{error::CommandError, prompt};
use std::iter::successors;

use super::*;
use colored::*;
use mipsy_parser::*;
use mipsy_lib::compile::Breakpoint;

enum BpState {
    Enable,
    Disable,
    Toggle,
}

#[derive(PartialEq)]
enum MipsyArgType {
    Immediate,
    Label,
    Id,
}

pub(crate) fn breakpoint_command() -> Command {
    command(
        "breakpoint",
        vec!["br", "brk", "break"],
        vec!["subcommand"],
        vec![],
        &format!(
            "manage breakpoints ({} to list subcommands)",
            "help breakpoint".bold()
        ),
        |state, label, args| {
            if label == "__help__" && args.is_empty() {
                return Ok(
                    get_long_help()
                )
            }

            // TODO(joshh): match on label for breakpoints aliases?
            match &*args[0] {
                "l" | "list" =>
                    breakpoint_list  (state, label, &args[1..]),
                "i" | "in" | "ins" | "insert" | "add" =>
                    breakpoint_insert(state, label, &args[1..], false),
                "del" | "delete" | "rm" | "remove" =>
                    breakpoint_insert(state, label, &args[1..], true),
                "tmp" | "temp" | "temporary" =>
                    breakpoint_insert(state, label,  args,      false),
                "e" | "enable" =>
                    breakpoint_toggle(state, label,  args, BpState::Enable),
                "d" | "disable" =>
                    breakpoint_toggle(state, label,  args, BpState::Disable),
                "t" | "toggle" =>
                    breakpoint_toggle(state, label,  args, BpState::Toggle),
                 _ =>
                    breakpoint_insert(state, label,  args, false),
            }
        }
    )
}

fn get_long_help() -> String {
    format!(
        "A collection of commands for managing breakpoints. Available {10}s are:\n\n\
         {0} {2}    : insert/delete a breakpoint\n\
         {1} {3}\n\
         {0} {11} : insert a temporary breakpoint that deletes itself after being hit\n\
         {0} {5}    : enable/disable an existing breakpoint\n\
         {1} {6}\n\
         {1} {7}\n\
         {0} {4}      : list currently set breakpoints\n\n\
         {8} {9} will provide more information about the specified subcommand.
        ",
        "breakpoint".yellow().bold(),
        "          ".yellow().bold(),
        "insert".purple(),
        "delete".purple(),
        "list".purple(),
        "enable".purple(),
        "disable".purple(),
        "toggle".purple(),
        "help breakpoint".bold(),
        "<subcommand>".purple().bold(),
        "<subcommand>".purple(),
        "temporary".purple(),
    )
}

fn breakpoint_insert(state: &mut State, label: &str, mut args: &[String], remove: bool) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(
            format!(
                "Usage: {10} {11} {12} {2}\n\
                 {0}s or {1}s a breakpoint at the specified {2}.\n\
                 {2} may be: a decimal address (`4194304`), a hex address (`{3}400000`), or a label (`{4}`).\n\
                 If you are removing a breakpoint, you can also use its id (`{5}`).\n\
                 {6} must be `i`, `in`, `ins`, `insert`, or `add` to insert the breakpoint, or\n\
            \x20             `del`, `delete`, `rm` or `remove` to remove the breakpoint.\n\
                 If {6} is none of these option, it defaults to inserting a breakpoint at {6}.\n\
                 When running or stepping through your program, a breakpoint will cause execution to\n\
                 pause temporarily, allowing you to debug the current state.\n\
                 May error if provided a label that doesn't exist.\n\
                 If temporary is provided as the second argument, the breakpoint will be created as a\n\
                 temporary breakpoint, which automatically deletes itself after being hit.
              \n{7}{8} you can also use the `{9}` MIPS instruction in your program's code!",
                "<insert>".magenta(),
                "<delete>".magenta(),
                "<address>".magenta(),
                "0x".yellow(),
                "main".yellow().bold(),
                "!3".blue(),
                "<subcommand>".magenta(),
                "tip".yellow().bold(),
                ":".bold(),
                "break".bold(),
                "breakpoint".yellow().bold(),
                "{insert, delete}".purple(),
                "temporary?".purple(),
            )
        )
    }

    let temporary = !args.is_empty() &&
        matches!(args[0].as_ref(), "t" | "tmp" | "temp" | "temporary");
    if temporary { args = &args[1..]; }

    if args.is_empty() {
        return Err(
            generate_err(
                CommandError::MissingArguments {
                    args: vec!["addr".to_string()],
                    instead: args.to_vec(),
                },
                "rm",
            )
        );
    }

    let (addr, arg_type) = parse_breakpoint_arg(state, &args[0])?;

    if addr % 4 != 0 {
        prompt::error_nl(format!("address 0x{:08x} should be word-aligned", addr));
        return Ok("".into());
    }

    let binary = state.binary.as_mut().ok_or(CommandError::MustLoadFile)?;

    let id;
    let action = if remove {
        if let Some(bp) = binary.breakpoints.remove(&addr) {
            id = bp.id;
            "removed"
        } else {
            prompt::error_nl(format!(
                "breakpoint at {} doesn't exist",
                match arg_type {
                    MipsyArgType::Immediate => args[0].white(),
                    MipsyArgType::Label     => args[0].yellow().bold(),
                    MipsyArgType::Id        => args[0].blue(),
                }
            ));
            return Ok("".into());
        }
    } else if !binary.breakpoints.contains_key(&addr) {
        id = binary.generate_breakpoint_id();
        let mut bp = Breakpoint::new(id);
        if temporary {
            bp.commands.push(format!("breakpoint remove !{id}"))
        }
        binary.breakpoints.insert(addr, bp);

        "inserted"
    } else {
        prompt::error_nl(format!(
            "breakpoint at {} already exists",
            match arg_type {
                MipsyArgType::Immediate => args[0].white(),
                MipsyArgType::Label     => args[0].yellow().bold(),
                MipsyArgType::Id        => args[0].blue(),
            }
        ));
        return Ok("".into());
    };

    let label = match arg_type {
        MipsyArgType::Immediate => None,
        MipsyArgType::Label     => Some(&args[0]),
        MipsyArgType::Id        => {
            let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
            binary.labels.iter()
                .find(|(_, &_addr)| _addr == addr)
                .map(|(name, _)| name)
        },
    };

    if let Some(label) = label {
        prompt::success_nl(format!("breakpoint {} {} at {} (0x{:08x})", format!("!{}", id).blue(), action, label.yellow().bold(), addr));
    } else {
        prompt::success_nl(format!("breakpoint {} {} at 0x{:08x}",      format!("!{}", id).blue(), action, addr));
    }

    Ok("".into())
}

fn breakpoint_list(state: &State, label: &str, _args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(
            format!(
                "Lists currently set breakpoints.\n\
                 When running or stepping through your program, a breakpoint will cause execution to\n\
                 pause temporarily, allowing you to debug the current state.\n\
                 May error if provided a label that doesn't exist.\n\
               \n{}{} you can also use the `{}` mips instruction in your program's code!",
                 "tip".yellow().bold(),
                 ":".bold(),
                 "break".bold(),
            ),
        )
    }

    let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;

    if binary.breakpoints.is_empty() {
        prompt::error_nl("no breakpoints set");
        return Ok("".into());
    }

    let mut breakpoints = binary.breakpoints.iter()
            .map(|x| {
                let (&addr, bp) = x;
                let id = bp.id;
                (
                    (
                        id,
                        // TODO (joshh): replace with checked_log10 when
                        // https://github.com/rust-lang/rust/issues/70887 is stabilised
                        successors(Some(id), |&id| (id >= 10).then(|| id / 10)).count(),
                    ),
                    addr,
                    binary.labels.iter()
                        .find(|(_, &val)| val == addr)
                        .map(|(name, _)| name),
                    bp.enabled,
                )
            })
            .collect::<Vec<_>>();

    breakpoints.sort_by_key(|(_, addr, _, _)| *addr);

    let max_id_len = breakpoints.iter()
            .map(|&(id, _, _, _)| {
                id.1
            })
            .max()
            .unwrap_or(0);

    println!("\n{}", "[breakpoints]".green().bold());
    for (id, addr, text, enabled) in breakpoints {
        let disabled = match enabled {
            true  => "",
            false => " (disabled)"
        };

        match text {
            Some(name) => {
                println!("{}{}: {}{:08x} ({}){}", " ".repeat(max_id_len - id.1), id.0.to_string().blue(), "0x".magenta(), addr, name.yellow().bold(), disabled.bright_black());
            }
            None => {
                println!("{}{}: {}{:08x}{}",      " ".repeat(max_id_len - id.1), id.0.to_string().blue(), "0x".magenta(), addr, disabled.bright_black());
            }
        }
    }
    println!();

    Ok("".into())
}

fn breakpoint_toggle(state: &mut State, label: &str, mut args: &[String], enabled: BpState) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(
            format!(
                "Usage: {8} {9} {3}\n\
                 {0}s, {1}s, or {2}s a breakpoint at the specified {3}.\n\
                 {3} may be: a decimal address (`4194304`), a hex address (`{4}400000`),\n\
        \x20                 a label (`{5}`), or an id (`{6}`).\n\
                 Breakpoints that are disabled do not trigger when they are hit.\n\
                 Breakpoints caused by the `{7}` instruction in code cannot be disabled.
                ",
                "<enable>".purple(),
                "<disable>".purple(),
                "<toggle>".purple(),
                "<address>".purple(),
                "0x".yellow(),
                "main".yellow().bold(),
                "!3".blue(),
                "break".bold(),
                "breakpoint".yellow().bold(),
                "{enable, disable, toggle}".purple(),
            )
        )
    }

    if args.len() == 1 {
        return Err(
            generate_err(
                CommandError::MissingArguments {
                    args: vec!["addr".to_string()],
                    instead: args.to_vec(),
                },
                &args[0],
            )
        );
    }
    args = &args[1..];

    let (addr, arg_type) = parse_breakpoint_arg(state, &args[0])?;

    if addr % 4 != 0 {
        prompt::error_nl(format!("address 0x{:08x} should be word-aligned", addr));
        return Ok("".into());
    }

    let binary = state.binary.as_mut().ok_or(CommandError::MustLoadFile)?;

    let id;
    if let Some(br) = binary.breakpoints.get_mut(&addr) {
        id = br.id;
        br.enabled = match enabled {
            BpState::Enable  => true,
            BpState::Disable => false,
            BpState::Toggle  => !br.enabled,
        }
    } else {
        prompt::error_nl(format!(
            "breakpoint at {} doesn't exist",
            match arg_type {
                MipsyArgType::Immediate => args[0].white(),
                MipsyArgType::Label     => args[0].yellow().bold(),
                MipsyArgType::Id        => args[0].blue(),
            }
        ));
        return Ok("".into());
    }

    // already ruled out possibility of entry not existing
    let action = match binary.breakpoints.get(&addr).unwrap().enabled {
        true  => "enabled",
        false => "disabled",
    };

    let label = match arg_type {
        MipsyArgType::Immediate => None,
        MipsyArgType::Label     => Some(&args[0]),
        MipsyArgType::Id        => {
            let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
            binary.labels.iter()
                .find(|(_, &_addr)| _addr == addr)
                .map(|(name, _)| name)
        },
    };

    if let Some(label) = label {
        prompt::success_nl(format!("breakpoint {} {} at {} (0x{:08x})", format!("!{}", id).blue(), action, label.yellow().bold(), addr));
    } else {
        prompt::success_nl(format!("breakpoint {} {} at 0x{:08x}",      format!("!{}", id).blue(), action, addr));
    }

    Ok("".into())
}


fn generate_err(error: CommandError, command_name: impl Into<String>) -> CommandError {
    let mut help = String::from("help breakpoint");
    let command_name = command_name.into();
    if !command_name.is_empty() { help.push(' ') };

    CommandError::WithTip {
        error: Box::new(error),
        tip: format!("try `{}{}`", help.bold(), command_name.bold()),
    } 
}

fn parse_breakpoint_arg(state: &State, arg: &String) -> Result<(u32, MipsyArgType), CommandError> {
    let get_error = |expected: &str| generate_err(
        CommandError::BadArgument { arg: expected.magenta().to_string(), instead: arg.into() },
        &String::from(""),
    );

    let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;

    if let Some(id) = arg.strip_prefix('!') {
        let id: u32 = id.parse().map_err(|_| get_error("<id>"))?;
        let addr = binary.breakpoints.iter().find(|bp| bp.1.id == id)
                        .ok_or_else(|| CommandError::InvalidBpId { arg: arg.to_string() })?.0;

        return Ok((*addr, MipsyArgType::Id))
    }

    let arg = mipsy_parser::parse_argument(arg, state.config.tab_size)
            .map_err(|_| get_error("<addr>"))?;

    if let MpArgument::Number(MpNumber::Immediate(ref imm)) = arg {
        Ok(match imm {
            MpImmediate::I16(imm) => {
                (*imm as u32, MipsyArgType::Immediate)
            }
            MpImmediate::U16(imm) => {
                (*imm as u32, MipsyArgType::Immediate)
            }
            MpImmediate::I32(imm) => {
                (*imm as u32, MipsyArgType::Immediate)
            }
            MpImmediate::U32(imm) => {
                (*imm, MipsyArgType::Immediate)
            }
            MpImmediate::LabelReference(label) => {
                (
                    binary.get_label(label)
                        .map_err(|_| CommandError::UnknownLabel { label: label.to_string() })?,
                    MipsyArgType::Label,
                )
            }
        })
    } else {
        Err(get_error("<addr>"))
    }
}
