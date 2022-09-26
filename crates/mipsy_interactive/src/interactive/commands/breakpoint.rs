use crate::interactive::{error::CommandError, prompt};
use std::iter::successors;

use super::{*, commands::handle_commands};
use colored::*;
use mipsy_lib::compile::breakpoints::Breakpoint;
use mipsy_parser::*;

enum EnableOp {
    Enable,
    Disable,
    Toggle,
}

#[derive(PartialEq)]
enum InsertOp {
    Insert,
    Delete,
    Temporary,
}

enum MipsyArgType {
    LineNumber,
    Immediate,
    Label,
    Id,
}

pub(crate) fn breakpoint_command() -> Command {
    let subcommands: Vec<Command> = vec![
        command(
            "list", 
            vec!["l"],
            vec![], vec![], vec![], "",
            |_, state, label, args| breakpoint_list(state, label, args)
        ),
        command(
            "insert", 
            vec!["i", "in", "ins", "add"],
            vec![], vec![], vec![], "",
            |_, state, label, args| breakpoint_insert(state, label, args, InsertOp::Insert)
        ),
        command(
            "remove", 
            vec!["del", "delete", "r", "rm"],
            vec![], vec![], vec![], "",
            |_, state, label, args| breakpoint_insert(state, label, args, InsertOp::Delete)
        ),
        command(
            "temporary", 
            vec!["tmp", "temp"],
            vec![], vec![], vec![], "",
            |_, state, label, args| breakpoint_insert(state, label, args, InsertOp::Temporary)
        ),
        command(
            "enable", 
            vec!["e"],
            vec![], vec![], vec![], "",
            |_, state, label, args| breakpoint_toggle(state, label, args, EnableOp::Enable)
        ),
        command(
            "disable", 
            vec!["d"],
            vec![], vec![], vec![], "",
            |_, state, label, args| breakpoint_toggle(state, label, args, EnableOp::Disable)
        ),
        command(
            "toggle", 
            vec!["t"],
            vec![], vec![], vec![], "",
            |_, state, label, args| breakpoint_toggle(state, label, args, EnableOp::Toggle)
        ),
        command(
            "ignore", 
            vec![],
            vec![], vec![], vec![], "",
            |_, state, label, args| breakpoint_ignore(state, label, args)
        ),
        command(
            "commands", 
            vec!["com", "comms", "cmd", "cmds", "command"],
            vec![], vec![], vec![], "",
            |_, state, label, args| breakpoint_commands(state, label, args)
        ),
    ];

    command(
        "breakpoint",
        vec!["bp", "br", "brk", "break"],
        vec!["subcommand"],
        vec![],
        subcommands,
        &format!(
            "manage breakpoints ({} to list subcommands)",
            "help breakpoint".bold()
        ),
        |cmd, state, label, args| {
            if label == "__help__" && args.is_empty() {
                return Ok(
                    get_long_help()
                )
            }

            let cmd = cmd.subcommands.iter().find(|c| c.name == args[0] || c.aliases.contains(&args[0]));
            match cmd {
                None if label == "__help__" => Ok(get_long_help()),
                Some(cmd) => cmd.exec(state, label, &args[1..]),
                None => breakpoint_insert(state, label, args, InsertOp::Insert),
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
         {0} {13}  : attach commands to a breakpoint\n\
         {0} {5}    : enable/disable an existing breakpoint\n\
         {1} {6}\n\
         {1} {7}\n\
         {0} {12}    : ignore a breakpoint for a specified number of hits\n\
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
        "ignore".purple(),
        "commands".purple(),
    )
}

fn breakpoint_insert(state: &mut State, label: &str, args: &[String], op: InsertOp) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(
            format!(
                "Usage: {10} {11} {2}\n\
                 {0}s or {1}s a breakpoint at the specified {2}.\n\
                 {2} may be: a decimal address (`4194304`), a hex address (`{3}400000`),\n\
            \x20             a label (`{4}`), or a line number (`:14`, `prog.s:7`).\n\
                 If you are removing a breakpoint, you can also use its id (`{5}`).\n\
                 {6} must be `i`, `in`, `ins`, `insert`, or `add` to insert the breakpoint, or\n\
            \x20             `del`, `delete`, `rm` or `remove` to remove the breakpoint.\n\
                 If {12}, `tmp`, or `temp` is provided as the {6}, the breakpoint will\n\
                 be created as a temporary breakpoint, which automatically deletes itself after being hit.\n\
                 If {6} is none of these option, it defaults to inserting a breakpoint at {6}.\n\
                 When running or stepping through your program, a breakpoint will cause execution to\n\
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
                "breakpoint".yellow().bold(),
                "{insert, delete, temporary}".purple(),
                "<temporary>".purple(),
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
                match op {
                    InsertOp::Insert    => "insert",
                    InsertOp::Delete    => "delete",
                    InsertOp::Temporary => "temporary",
                },
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
    let action = if op == InsertOp::Delete {
        if let Some(bp) = binary.breakpoints.remove(&addr) {
            id = bp.id;
            "removed"
        } else {
            prompt::error_nl(format!(
                "breakpoint at {} doesn't exist",
                match arg_type {
                    MipsyArgType::LineNumber => args[0].as_str().into(),
                    MipsyArgType::Immediate  => args[0].white(),
                    MipsyArgType::Label      => args[0].yellow().bold(),
                    MipsyArgType::Id         => args[0].blue(),
                }
            ));
            return Ok("".into());
        }
    } else if !binary.breakpoints.contains_key(&addr) {
        id = binary.generate_breakpoint_id();
        let mut bp = Breakpoint::new(id);
        if op == InsertOp::Temporary {
            bp.commands.push(format!("breakpoint remove !{id}"))
        }
        binary.breakpoints.insert(addr, bp);

        "inserted"
    } else {
        prompt::error_nl(format!(
            "breakpoint at {} already exists",
            match arg_type {
                MipsyArgType::LineNumber => args[0].as_str().into(),
                MipsyArgType::Immediate  => args[0].white(),
                MipsyArgType::Label      => args[0].yellow().bold(),
                MipsyArgType::Id         => args[0].blue(),
            }
        ));
        return Ok("".into());
    };

    let label = match arg_type {
        MipsyArgType::Immediate => None,
        MipsyArgType::Label     => Some(&args[0]),
        MipsyArgType::Id | MipsyArgType::LineNumber => {
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
                    bp,
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
    for (id, addr, text, bp) in breakpoints {
        let disabled = match bp.enabled {
            true  => "",
            false => " (disabled)",
        };

        let ignored = match bp.ignore_count {
            0 => "".to_string(),
            i => format!(" (ignored for the next {} hits)", i.to_string().bold()),
        };

        match text {
            Some(name) => {
                println!("{}{}: {}{:08x} ({}){}{}", " ".repeat(max_id_len - id.1), id.0.to_string().blue(), "0x".magenta(), addr, name.yellow().bold(), disabled.bright_black(), ignored);
            }
            None => {
                println!("{}{}: {}{:08x}{}{}",      " ".repeat(max_id_len - id.1), id.0.to_string().blue(), "0x".magenta(), addr, disabled.bright_black(), ignored);
            }
        }
    }
    println!();

    Ok("".into())
}

fn breakpoint_toggle(state: &mut State, label: &str, args: &[String], op: EnableOp) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(
            format!(
                "Usage: {8} {9} {3}\n\
                 {0}s, {1}s, or {2}s a breakpoint at the specified {3}.\n\
                 {3} may be: a decimal address (`4194304`), a hex address (`{4}400000`),\n\
        \x20                 a label (`{5}`), a line number (`:14`, `prog.s:7`), or an id (`{6}`).\n\
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

    if args.is_empty() {
        return Err(
            generate_err(
                CommandError::MissingArguments {
                    args: vec!["addr".to_string()],
                    instead: args.to_vec(),
                },
                match op {
                    EnableOp::Enable  => "enable",
                    EnableOp::Disable => "disable",
                    EnableOp::Toggle  => "toggle",
                },
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
    if let Some(br) = binary.breakpoints.get_mut(&addr) {
        id = br.id;
        br.enabled = match op {
            EnableOp::Enable  => true,
            EnableOp::Disable => false,
            EnableOp::Toggle  => !br.enabled,
        }
    } else {
        prompt::error_nl(format!(
            "breakpoint at {} doesn't exist",
            match arg_type {
                MipsyArgType::LineNumber => args[0].as_str().into(),
                MipsyArgType::Immediate  => args[0].white(),
                MipsyArgType::Label      => args[0].yellow().bold(),
                MipsyArgType::Id         => args[0].blue(),
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
        MipsyArgType::Id | MipsyArgType::LineNumber => {
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

fn breakpoint_ignore(state: &mut State, label: &str, mut args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(
            format!(
                "Usage: {6} {7} {1} {0}\n\
                 {7}s a breakpoint at the specified {1} for the next {0} hits.\n\
                 {1} may be: a decimal address (`4194304`), a hex address (`{2}400000`),\n\
        \x20                 a label (`{3}`), a line number (`:14`, `prog.s:7`), or an id (`{4}`).\n\
                 Breakpoints that are ignored do not trigger when they are hit.\n\
                 Breakpoints caused by the `{5}` instruction in code cannot ignored.
                ",
                "<ignore count>".purple(),
                "<address>".purple(),
                "0x".yellow(),
                "main".yellow().bold(),
                "!3".blue(),
                "break".bold(),
                "breakpoint".yellow().bold(),
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

    let (addr, arg_type) = parse_breakpoint_arg(state, &args[0])?;

    if addr % 4 != 0 {
        prompt::error_nl(format!("address 0x{:08x} should be word-aligned", addr));
        return Ok("".into());
    }

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

    let binary = state.binary.as_mut().ok_or(CommandError::MustLoadFile)?;

    if let Some(br) = binary.breakpoints.get_mut(&addr) {
        br.ignore_count = ignore_count;
        prompt::success_nl(format!("skipping breakpoint {} {} times", format!("!{}", br.id).blue(), ignore_count.to_string().yellow()));
    } else {
        prompt::error_nl(format!(
            "breakpoint at {} doesn't exist",
            match arg_type {
                MipsyArgType::LineNumber => args[0].as_str().into(),
                MipsyArgType::Immediate  => args[0].white(),
                MipsyArgType::Label      => args[0].yellow().bold(),
                MipsyArgType::Id         => args[0].blue(),
            }
        ));
    }

    Ok("".into())
}

fn breakpoint_commands(state: &mut State, label: &str, args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(
            format!(
                "Takes in a list of commands seperated by newlines,\n\
                 and attaches the commands to the specified {0}.\n\
                 If no breakpoint is specified, the most recently created breakpoint is chosen.\n\
                 Whenever that breakpoint is hit, the commands will automatically be executed\n\
                 in the provided order.\n\
                 The list of commands can be ended using the {1} command, EOF, or an empty line.\n\
                 To view the commands attached to a particular breakpoint,\n\
                 use {2} {0}
                ",
                "<breakpoint id>".purple(),
                "end".yellow().bold(),
                "commands list".bold().yellow(),
            )
        )
    }

    let binary = state.binary.as_mut().ok_or(CommandError::MustLoadFile)?;
    state.confirm_exit = true;
    handle_commands(args, &mut binary.breakpoints)
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
                        .ok_or(CommandError::InvalidBpId { arg: arg.to_string() })?.0;

        return Ok((*addr, MipsyArgType::Id))
    }

    if arg.contains(':') {
        // parts contains at least 2 elements
        let mut parts = arg.split(':');
        let mut file = parts.next().unwrap();
        if file.is_empty() {
            let mut filenames = binary.line_numbers.values()
                    .map(|(filename, _)| filename);
            file = filenames.next().unwrap();
            if !filenames.all(|f| f.as_ref() == file) {
                return Err(CommandError::MustSpecifyFile);
            }
        }

        let line_number: u32 = parts.next().unwrap().parse().map_err(|_| get_error("<line number>"))?;
        let mut lines = binary.line_numbers.iter()
            .filter(|(_, (filename, _))| filename.as_ref() == file).collect::<Vec<_>>();
        lines.sort_unstable_by(|a, b| a.1.1.cmp(&b.1.1));

        // use first line after the specified line that contains an instruction
        let addr = lines.iter()
            .find(|(_, &(_, _line_number))| _line_number >= line_number)
            .ok_or_else(|| CommandError::LineDoesNotExist { line_number })?.0;

        return Ok((*addr, MipsyArgType::LineNumber))
    }

    let arg = mipsy_parser::parse_argument(arg, state.config.tab_size)
            .map_err(|_| get_error("<addr>"))?;

    if let MpArgument::Number(MpNumber::Immediate(ref imm)) = arg {
        Ok(match imm {
            MpImmediate::I16(imm) => (*imm as u32, MipsyArgType::Immediate),
            MpImmediate::U16(imm) => (*imm as u32, MipsyArgType::Immediate),
            MpImmediate::I32(imm) => (*imm as u32, MipsyArgType::Immediate),
            MpImmediate::U32(imm) => (*imm, MipsyArgType::Immediate),
            MpImmediate::LabelReference(label) =>
                (
                    binary.get_label(label)
                        .map_err(|_| CommandError::UnknownLabel { label: label.to_string() })?,
                    MipsyArgType::Label,
                ),
        })
    } else {
        Err(get_error("<addr>"))
    }
}
