use colored::Colorize;
use mipsy_lib::Binary;
use rustyline::error::ReadlineError;

use crate::{interactive::{editor, error::CommandError}, prompt};

use super::*;

pub(crate) fn commands_command() -> Command {
    command(
        "commands",
        vec![],
        vec![],
        vec!["breakpoint id"],
        "attach commands to a breakpoint",
        |state, label, mut args| {
            let get_error = |expected: &str, instead: &str| generate_err(
                CommandError::BadArgument { arg: expected.magenta().to_string(), instead: instead.into() },
                &String::from(""),
            );

            if label == "__help__" {
                return Ok(
                    "TODO: commands help".into()
                    // make sure to talk about optional specification
                    // and make the end syntax clear
                )
            }

            let binary = state.binary.as_mut().ok_or(CommandError::MustLoadFile)?;

            let list = !args.is_empty() &&
                matches!(args[0].as_ref(), "l" | "list");
            if list { args = &args[1..]; }

            let id: u32 = if args.is_empty() {
                binary.breakpoints.values()
                    .map(|bp| bp.id)
                    .fold(u32::MIN, |x, y| x.max(y))
            } else if let Some(id) = args[0].strip_prefix('!') {
                id.parse().map_err(|_| get_error("<id>", &args[0]))?
            } else {
                return Err(get_error("<id>", &args[0]))
            };

            state.confirm_exit = true;
            if list {
                list_commands(binary, id)
            } else {
                add_commands(binary, id)
            }
        }
    )
}

fn add_commands(binary: &mut Binary, id: u32) -> CommandResult<String> {
    println!("[mipsy] enter commands seperated by newlines\n\
              [mipsy] to run whenever breakpoint {} is hit", format!("!{id}").blue());
    println!("[mipsy] use an empty line or the command {} to finish", "end".bold().yellow());

    let commands;
    if let Some(br) = binary.breakpoints.iter_mut().find(|bp| bp.1.id == id) {
        commands = &mut br.1.commands;

        // TODO: decide on behavious when commands already exist
        commands.clear();
    } else {
        prompt::error_nl(format!(
            "breakpoint at {} doesn't exist",
            format!("!{id}").blue(),
        ));
        return Ok("".into());
    }

    let mut rl = editor();
    loop {
        let readline = rl.readline("");

        match readline {
            Ok(line) => {
                if line.is_empty() || line == "\n" || line == "end" {
                    break;
                }

                commands.push(line);
            }
            Err(ReadlineError::Interrupted) => {
                std::process::exit(0);
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }

    prompt::success_nl(format!("commands attached to breakpoint {}", format!("!{id}").blue()));
    Ok("".into())
}

fn list_commands(binary: &mut Binary, id: u32) -> CommandResult<String> {
    println!("[mipsy] commands for breakpoint {}:", format!("!{id}").blue());

    if let Some(br) = binary.breakpoints.iter_mut().find(|bp| bp.1.id == id) {
        let commands = &mut br.1.commands;
        commands.iter().for_each(|command| println!("{command}"));
    } else {
        prompt::error_nl(format!(
            "breakpoint at {} doesn't exist",
            format!("!{id}").blue(),
        ));
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
