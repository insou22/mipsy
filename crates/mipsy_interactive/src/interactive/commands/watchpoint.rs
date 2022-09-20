use crate::interactive::{error::CommandError, prompt, RegisterAction, Watchpoint};
use std::{iter::successors, str::FromStr};

use super::*;
use colored::*;
use mipsy_lib::Register;

enum WpState {
    Enable,
    Disable,
    Toggle,
}

#[derive(PartialEq)]
enum MipsyArgType {
    Register,
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
         {0} {11} : insert a temporary watchpoint that deletes itself after being hit\n\
         {0} {5}    : enable/disable an existing watchpoint\n\
         {1} {6}\n\
         {1} {7}\n\
         {0} {12}    : ignore a watchpoint for a specified number of hits\n\
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
        "temporary".purple(),
        "ignore".purple(),
    )
}

fn watchpoint_insert(state: &mut State, label: &str, args: &[String], remove: bool) -> Result<String, CommandError> {
    if label == "__help__" {
        // TODO(joshh): fix help for watchpoints, remove temporary etc
        return Ok(
            format!(
                "Usage: {10} {11} {2}\n\
                 {0}s or {1}s a watchpoint at the specified {2}.\n\
                 {2} may be: a decimal address (`4194304`), a hex address (`{3}400000`), or a label (`{4}`).\n\
                 If you are removing a watchpoint, you can also use its id (`{5}`).\n\
                 {6} must be `i`, `in`, `ins`, `insert`, or `add` to insert the watchpoint, or\n\
            \x20             `del`, `delete`, `rm` or `remove` to remove the watchpoint.\n\
                 If {12}, `tmp`, or `temp` is provided as the {6}, the watchpoint will\n\
                 be created as a temporary watchpoint, which automatically deletes itself after being hit.\n\
                 If {6} is none of these option, it defaults to inserting a watchpoint at {6}.\n\
                 When running or stepping through your program, a watchpoint will cause execution to\n\
                 pause temporarily, allowing you to debug the current state.\n\
                 May error if provided a label that doesn't exist.\n\
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
                "watchpoint".yellow().bold(),
                "{insert, delete, temporary}".purple(),
                "<temporary>".purple(),
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

    let (register, arg_type) = parse_watchpoint_arg(state, &args[0])?;

    let args = &args[1..];
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

    let action = match args[0].as_str() {
        "r" | "read"  => RegisterAction::ReadOnly,
        "w" | "write" => RegisterAction::WriteOnly,
        "rw" | "r/w" | "r+w" | "w/r" | "w+r" | "read/write" | "read+write"
            => RegisterAction::ReadWrite,
        _ => return Err(
                generate_err(CommandError::BadArgument {
                    arg: "action".to_owned(),
                    instead: args[0].clone(),
                },
                "rm",
            )
        )
    };

    let id;
    let wp_action = if remove {
        if let Some(wp) = state.watchpoints.remove(&register) {
            id = wp.id;
            "removed"
        } else {
            prompt::error_nl(format!(
                "watchpoint at {} doesn't exist",
                match arg_type {
                    MipsyArgType::Register => register.to_string().as_str().into(),
                    MipsyArgType::Id       => args[0].blue(),
                }
            ));
            return Ok("".into());
        }
    } else {
        let task = if state.watchpoints.contains_key(&register) { "updated" } else { "inserted" };
        id = state.generate_watchpoint_id();
        let wp = Watchpoint::new(id, action);
        state.watchpoints.insert(register, wp);

        task
    };

    prompt::success_nl(format!("watchpoint {} {} for {} ({})",
        format!("!{}", id).blue(),
        wp_action,
        register,
        action
    ));

    Ok("".into())
}

fn watchpoint_list(state: &State, label: &str, _args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(
            format!(
                "Lists currently set watchpoints.\n\
                 When running or stepping through your program, a watchpoint will cause execution to\n\
                 pause temporarily, allowing you to debug the current state.\n\
                 May error if provided a label that doesn't exist.",
            ),
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
        let (register, wp) = wp;
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
            register,
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
                "Usage: {8} {9} {3}\n\
                 {0}s, {1}s, or {2}s a watchpoint at the specified {3}.\n\
                 {3} may be: a decimal address (`4194304`), a hex address (`{4}400000`),\n\
        \x20                 a label (`{5}`), or an id (`{6}`).\n\
                 watchpoints that are disabled do not trigger when they are hit.\n\
                 watchpoints caused by the `{7}` instruction in code cannot be disabled.
                ",
                "<enable>".purple(),
                "<disable>".purple(),
                "<toggle>".purple(),
                "<address>".purple(),
                "0x".yellow(),
                "main".yellow().bold(),
                "!3".blue(),
                "break".bold(),
                "watchpoint".yellow().bold(),
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

    let (register, arg_type) = parse_watchpoint_arg(state, &args[0])?;

    let id;
    if let Some(wp) = state.watchpoints.get_mut(&register) {
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
                MipsyArgType::Register => register.to_string().as_str().into(),
                MipsyArgType::Id       => args[0].blue(),
            }
        ));
        return Ok("".into());
    }

    // already ruled out possibility of entry not existing
    let action = match state.watchpoints.get(&register).unwrap().enabled {
        true  => "enabled",
        false => "disabled",
    };

    prompt::success_nl(format!("watchpoint {} {} for {} ({})",
        format!("!{}", id).blue(),
        action,
        register,
        action
    ));

    Ok("".into())
}

fn watchpoint_ignore(state: &mut State, label: &str, mut args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(
            format!(
                "Usage: {6} {7} {1} {0}\n\
                 {7}s a watchpoint at the specified {1} for the next {0} hits.\n\
                 {1} may be: a decimal address (`4194304`), a hex address (`{2}400000`),\n\
        \x20                 a label (`{3}`), or an id (`{4}`).\n\
                 watchpoints that are ignored do not trigger when they are hit.\n\
                 watchpoints caused by the `{5}` instruction in code cannot ignored.
                ",
                "<ignore count>".purple(),
                "<address>".purple(),
                "0x".yellow(),
                "main".yellow().bold(),
                "!3".blue(),
                "break".bold(),
                "watchpoint".yellow().bold(),
                "ignore".purple(),
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

    let (register, arg_type) = parse_watchpoint_arg(state, &args[0])?;

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

    if let Some(wp) = state.watchpoints.get_mut(&register) {
        wp.ignore_count = ignore_count;
        prompt::success_nl(format!("skipping watchpoint {} {} times", format!("!{}", wp.id).blue(), ignore_count.to_string().yellow()));
    } else {
        prompt::error_nl(format!(
            "watchpoint at {} doesn't exist",
            match arg_type {
                MipsyArgType::Register => register.to_string().as_str().into(),
                MipsyArgType::Id       => args[0].blue(),
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

fn parse_watchpoint_arg(state: &State, arg: &String) -> Result<(Register, MipsyArgType), CommandError> {
    let get_error = |expected: &str| generate_err(
        CommandError::BadArgument { arg: expected.magenta().to_string(), instead: arg.into() },
        &String::from(""),
    );

    if let Some(id) = arg.strip_prefix('!') {
        let id: u32 = id.parse().map_err(|_| get_error("<id>"))?;
        let register = state.watchpoints.iter().find(|wp| wp.1.id == id)
                        .ok_or_else(|| CommandError::InvalidBpId { arg: arg.to_string() })?.0;

        return Ok((*register, MipsyArgType::Id))
    }

    let arg = arg.strip_prefix('$').unwrap_or(arg);
    let register = Register::from_str(arg)
            .map_err(|_| get_error("<register>"))?;

    Ok((register, MipsyArgType::Register))
}
