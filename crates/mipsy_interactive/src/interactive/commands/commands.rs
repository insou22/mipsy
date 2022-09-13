use colored::Colorize;
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
        |state, label, args| {
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

            // TODO: give instructions
            // TODO: decide on behavious when commands already exist
            // println!("{} ")

            let id: u32 = if args.is_empty() {
                state.breakpoints.values()
                    .map(|bp| bp.breakpoint.id)
                    .fold(u32::MIN, |x, y| x.max(y))
            } else if let Some(id) = args[0].strip_prefix('!') {
                id.parse().map_err(|_| get_error("<id>", &args[0]))?
            } else {
                return Err(get_error("<id>", &args[0]))
            };

            let commands;
            if let Some(br) = state.breakpoints.iter_mut().find(|bp| bp.1.breakpoint.id == id) {
                commands = &mut br.1.commands;
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
                            state.confirm_exit = true;
                            break;
                        }

                        commands.push(line);
                    }
                    Err(ReadlineError::Interrupted) => {}
                    Err(ReadlineError::Eof) => {
                        std::process::exit(0);
                    }
                    Err(err) => {
                        println!("Error: {:?}", err);
                        break;
                    }
                }
            }

            Ok("".into())
        }
    )
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
