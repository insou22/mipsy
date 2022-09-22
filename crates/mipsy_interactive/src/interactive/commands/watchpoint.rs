use crate::interactive::{error::CommandError, prompt};
use std::{iter::successors, str::FromStr};

use super::*;
use colored::*;
use mipsy_lib::Register;
use mipsy_parser::{MpArgument, MpNumber, MpImmediate};

enum WpState {
    Enable,
    Disable,
    Toggle,
}

#[derive(PartialEq)]
enum MipsyArgType {
    Target,
    Id,
}

pub(crate) fn watchpoint_command() -> Command {
    command(
        "watchpoint",
        vec!["w", "wa", "wp", "watch"],
        vec!["subcommand"],
        vec![],
        &format!(
            "manage watchpoints ({} to list subcommands)",
            "help watchpoint".bold()
        ),
        |state, label, args| {
            if label == "__help__" && args.is_empty() {
                return Ok(
                    get_long_help()
                )
            }

            // TODO(joshh): match on label for watchpoints aliases?
            match &*args[0] {
                "l" | "list" =>
                    watchpoint_list  (state, label, &args[1..]),
                "i" | "in" | "ins" | "insert" | "add" =>
                    watchpoint_insert(state, label, &args[1..], false),
                "del" | "delete" | "rm" | "remove" =>
                    watchpoint_insert(state, label, &args[1..], true),
                "e" | "enable" =>
                    watchpoint_toggle(state, label,  args, WpState::Enable),
                "d" | "disable" =>
                    watchpoint_toggle(state, label,  args, WpState::Disable),
                "t" | "toggle" =>
                    watchpoint_toggle(state, label,  args, WpState::Toggle),
                "ignore" =>
                    watchpoint_ignore(state, label, &args[1..]),
                _ if label != "__help__" =>
                    watchpoint_insert(state, label,  args, false),
                _ =>
                    Ok(get_long_help()),
            }
        }
    )
}

fn get_long_help() -> String {
    format!(
        "A collection of commands for managing watchpoints. Available {10}s are:\n\n\
         {0} {2}    : insert/delete a watchpoint\n\
         {1} {3}\n\
         {0} {5}    : enable/disable an existing watchpoint\n\
         {1} {6}\n\
         {1} {7}\n\
         {0} {11}    : ignore a watchpoint for a specified number of hits\n\
         {0} {4}      : list currently set watchpoints\n\n\
         {8} {9} will provide more information about the specified subcommand.
        ",
        "watchpoint".yellow().bold(),
        "          ".yellow().bold(),
        "insert".purple(),
        "delete".purple(),
        "list".purple(),
        "enable".purple(),
        "disable".purple(),
        "toggle".purple(),
        "help watchpoint".bold(),
        "<subcommand>".purple().bold(),
        "<subcommand>".purple(),
        "ignore".purple(),
    )
}

fn watchpoint_insert(state: &mut State, label: &str, args: &[String], remove: bool) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(
            format!(
                "Usage: {5} {6} {2} {7}\n\
                 {0}s or {1}s a watchpoint at the specified {2}.\n\
                 {2} may be: a register name (`$t0`, `t0`), a register number (`$14`, `14`),\n\
                 a decimal address (`4194304`), a hex address (`{8}400000`), or a label (`{9}`).\n\
                 If you are removing a watchpoint, you can also use its id (`{3}`).\n\
                 {4} must be `i`, `in`, `ins`, `insert`, or `add` to insert the watchpoint, or\n\
            \x20             `del`, `delete`, `rm` or `remove` to remove the watchpoint.\n\
                 If {4} is none of these option, it defaults to inserting a watchpoint at {4}.\n\
                 When running or stepping through your program, a watchpoint will cause execution to\n\
                 pause temporarily when the specified register is read from or written to,\n\
                 allowing you to debug the current state.\n\
                 May error if provided a {2} that doesn't exist.",
                "<insert>".magenta(),
                "<delete>".magenta(),
                "<target>".magenta(),
                "!3".blue(),
                "<subcommand>".magenta(),
                "watchpoint".yellow().bold(),
                "{insert, delete}".purple(),
                "{read, write, read/write}".purple(),
                "0x".yellow(),
                "main".yellow().bold(),
            )
        )
    }

    if args.is_empty() {
        return Err(
            generate_err(
                CommandError::MissingArguments {
                    args: vec!["register".to_string()],
                    instead: args.to_vec(),
                },
                "rm",
            )
        );
    }

    let (target, arg_type) = parse_watchpoint_arg(state, &args[0])?;
    let args = &args[1..];

    let id;
    // this should always be overwritten but the compiler doesn't know that
    let mut action = TargetAction::ReadWrite;
    let wp_action = if remove {
        if let Some(wp) = state.watchpoints.remove(&target) {
            id = wp.id;
            "removed"
        } else {
            prompt::error_nl(format!(
                "watchpoint at {} doesn't exist",
                match arg_type {
                    MipsyArgType::Target => target.to_string().as_str().into(),
                    MipsyArgType::Id     => args[0].blue(),
                }
            ));
            return Ok("".into());
        }
    } else {
        if args.is_empty() {
            return Err(
                generate_err(
                    CommandError::MissingArguments {
                        args: vec!["action".to_string()],
                        instead: args.to_vec(),
                    },
                    "rm",
                )
            );
        }

        action = match args[0].as_str() {
            "r" | "read"  => TargetAction::ReadOnly,
            "w" | "write" => TargetAction::WriteOnly,
            "rw" | "r/w" | "r+w" | "w/r" | "w+r" | "read/write" | "read+write"
                => TargetAction::ReadWrite,
            _ => return Err(
                    generate_err(CommandError::BadArgument {
                        arg: "action".to_owned(),
                        instead: args[0].clone(),
                    },
                    "insert",
                )
            )
        };

        let task = if state.watchpoints.contains_key(&target) { "updated" } else { "inserted" };
        id = state.generate_watchpoint_id();
        let wp = Watchpoint::new(id, action);
        state.watchpoints.insert(target, wp);

        task
    };

    if remove {
        prompt::success_nl(format!("watchpoint {} {} for {}",
            format!("!{}", id).blue(),
            wp_action,
            target,
        ));
    } else {
        prompt::success_nl(format!("watchpoint {} {} for {} ({})",
            format!("!{}", id).blue(),
            wp_action,
            target,
            action
        ));
    }

    Ok("".into())
}

fn watchpoint_list(state: &State, label: &str, _args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(
            format!(
                "Lists currently set watchpoints."
            )
        )
    }

    if state.watchpoints.is_empty() {
        prompt::error_nl("no watchpoints set");
        return Ok("".into());
    }

    let mut watchpoints = state.watchpoints.iter()
            .map(|wp| {
                let id = wp.1.id;
                (
                    (
                        id,
                        // TODO (joshh): replace with checked_log10 when
                        // https://github.com/rust-lang/rust/issues/70887 is stabilised
                        successors(Some(id), |&id| (id >= 10).then(|| id / 10)).count(),
                    ),
                    wp,
                )
            })
            .collect::<Vec<_>>();

    watchpoints.sort_by_key(|(id, _)| id.0);

    let max_id_len = watchpoints.iter()
            .map(|(id, _)| {
                id.1
            })
            .max()
            .unwrap_or(0);

    println!("\n{}", "[watchpoints]".green().bold());
    for (id, wp) in watchpoints {
        let (target, wp) = wp;
        let disabled = match wp.enabled {
            true  => "",
            false => " (disabled)"
        };

        let ignored = match wp.ignore_count {
            0 => "".to_string(),
            i => format!(" (ignored for the next {} hits)", i.to_string().bold()),
        };

        println!("{}{}: {} ({}){}{}",
            " ".repeat(max_id_len - id.1), id.0.to_string().blue(),
            target,
            wp.action.to_string().purple(), disabled.bright_black(),
            ignored,
        );
    }
    println!();

    Ok("".into())
}

fn watchpoint_toggle(state: &mut State, label: &str, mut args: &[String], enabled: WpState) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(
            format!(
                "Usage: {5} {6} {3}\n\
                 {0}s, {1}s, or {2}s a watchpoint at the specified {3}.\n\
                 {3} may be: a register name (`$t0`, `t0`), a register number (`$14`, 14), or an id (`{4}`).\n\
                 a decimal address (`4194304`), a hex address (`{7}400000`), or a label (`{8}`).\n\
                 watchpoints that are disabled do not trigger when they are hit.",
                "<enable>".purple(),
                "<disable>".purple(),
                "<toggle>".purple(),
                "<target>".purple(),
                "!3".blue(),
                "watchpoint".yellow().bold(),
                "{enable, disable, toggle}".purple(),
                "0x".yellow(),
                "main".yellow().bold(),
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

    let (target, arg_type) = parse_watchpoint_arg(state, &args[0])?;

    let id;
    if let Some(wp) = state.watchpoints.get_mut(&target) {
        id = wp.id;
        wp.enabled = match enabled {
            WpState::Enable  => true,
            WpState::Disable => false,
            WpState::Toggle  => !wp.enabled,
        }
    } else {
        prompt::error_nl(format!(
            "watchpoint at {} doesn't exist",
            match arg_type {
                MipsyArgType::Target => target.to_string().as_str().into(),
                MipsyArgType::Id     => args[0].blue(),
            }
        ));
        return Ok("".into());
    }

    // already ruled out possibility of entry not existing
    let action = match state.watchpoints.get(&target).unwrap().enabled {
        true  => "enabled",
        false => "disabled",
    };

    prompt::success_nl(format!("watchpoint {} {} for {} ({})",
        format!("!{}", id).blue(),
        action,
        target,
        action
    ));

    Ok("".into())
}

fn watchpoint_ignore(state: &mut State, label: &str, mut args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(
            format!(
                "Usage: {3} {4} {1} {0}\n\
                 {4}s a watchpoint at the specified {1} for the next {0} hits.\n\
                 {1} may be: a register name (`$t0`, `t0`), a register number (`$14`, 14), or an id (`{2}`).\n\
                 a decimal address (`4194304`), a hex address (`{5}400000`), or a label (`{6}`).\n\
                 watchpoints that are ignored do not trigger when they are hit.\n\
                ",
                "<ignore count>".purple(),
                "<target>".purple(),
                "!3".blue(),
                "watchpoint".yellow().bold(),
                "ignore".purple(),
                "0x".yellow(),
                "main".yellow().bold(),
            )
        )
    }

    if args.is_empty() {
        return Err(
            generate_err(
                CommandError::MissingArguments {
                    args: vec!["addr".to_string()],
                    instead: args.to_vec(),
                },
                "ignore",
            )
        );
    }

    let (target, arg_type) = parse_watchpoint_arg(state, &args[0])?;

    args = &args[1..];
    if args.is_empty() {
        return Err(
            generate_err(
                CommandError::MissingArguments {
                    args: vec!["ignore count".to_string()],
                    instead: args.to_vec(),
                },
                "ignore",
            )
        );
    }

    let ignore_count: u32 = args[0].parse()
        .map_err(|_| generate_err(
            CommandError::BadArgument {
                arg: "<ignore count>".into(),
                instead: args[0].clone(),
            },
            ""
        ))?;

    if let Some(wp) = state.watchpoints.get_mut(&target) {
        wp.ignore_count = ignore_count;
        prompt::success_nl(format!("skipping watchpoint {} {} times", format!("!{}", wp.id).blue(), ignore_count.to_string().yellow()));
    } else {
        prompt::error_nl(format!(
            "watchpoint at {} doesn't exist",
            match arg_type {
                MipsyArgType::Target => target.to_string().as_str().into(),
                MipsyArgType::Id     => args[0].blue(),
            }
        ));
    }

    Ok("".into())
}

fn generate_err(error: CommandError, command_name: impl Into<String>) -> CommandError {
    let mut help = String::from("help watchpoint");
    let command_name = command_name.into();
    if !command_name.is_empty() { help.push(' ') };

    CommandError::WithTip {
        error: Box::new(error),
        tip: format!("try `{}{}`", help.bold(), command_name.bold()),
    } 
}

fn parse_watchpoint_arg(state: &State, arg: &String) -> Result<(WatchpointTarget, MipsyArgType), CommandError> {
    let get_error = |expected: &str| generate_err(
        CommandError::BadArgument { arg: expected.magenta().to_string(), instead: arg.into() },
        &String::from(""),
    );

    if let Some(id) = arg.strip_prefix('!') {
        let id: u32 = id.parse().map_err(|_| get_error("<id>"))?;
        let target = state.watchpoints.iter().find(|wp| wp.1.id == id)
                        .ok_or_else(|| CommandError::InvalidBpId { arg: arg.to_string() })?.0;

        return Ok((*target, MipsyArgType::Id))
    }

    let target = if let Ok(register) = Register::from_str(arg.strip_prefix('$').unwrap_or(arg)) {
        WatchpointTarget::Register(register)
    } else {
        let arg = mipsy_parser::parse_argument(arg, state.config.tab_size)
                .map_err(|_| get_error("<addr>"))?;

        if let MpArgument::Number(MpNumber::Immediate(ref imm)) = arg {
            WatchpointTarget::MemAddr(match imm {
                MpImmediate::I16(imm) =>  *imm as u32,
                MpImmediate::U16(imm) =>  *imm as u32,
                MpImmediate::I32(imm) =>  *imm as u32,
                MpImmediate::U32(imm) =>  *imm,
                MpImmediate::LabelReference(label) => {
                    let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
                    binary.get_label(label)
                        .map_err(|_| CommandError::UnknownLabel { label: label.to_string() })?
                }
            })
        } else {
            return Err(get_error("<addr>"))
        }
    };

    Ok((target, MipsyArgType::Target))
}
