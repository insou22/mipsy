use std::collections::HashMap;

use colored::Colorize;
use mipsy_lib::compile::breakpoints::Point;
use rustyline::error::ReadlineError;

use crate::{
    interactive::{editor, error::CommandError},
    prompt,
};

use super::*;

pub fn handle_commands<K, V: Point>(
    mut args: &[String],
    points: &mut HashMap<K, V>,
) -> Result<String, CommandError> {
    let get_error = |expected: &str, instead: &str| {
        generate_err(
            CommandError::BadArgument {
                arg: expected.magenta().to_string(),
                instead: instead.into(),
            },
            String::from(""),
        )
    };

    let list = !args.is_empty() && matches!(args[0].as_ref(), "l" | "list");
    if list {
        args = &args[1..];
    }

    let id: u32 = if args.is_empty() {
        points
            .values()
            .map(|bp| bp.get_id())
            .fold(u32::MIN, |x, y| x.max(y))
    } else if let Some(id) = args[0].strip_prefix('!') {
        id.parse().map_err(|_| get_error("<id>", &args[0]))?
    } else {
        return Err(get_error("<id>", &args[0]));
    };

    if list {
        list_commands(points, id)
    } else {
        add_commands(points, id)
    }
}

fn add_commands<K, V: Point>(points: &mut HashMap<K, V>, id: u32) -> CommandResult<String> {
    let commands;
    if let Some(br) = points.iter_mut().find(|bp| bp.1.get_id() == id) {
        commands = br.1.get_commands();

        // TODO: decide on behavious when commands already exist
        commands.clear();
    } else {
        prompt::error_nl(format!(
            "breakpoint at {} doesn't exist",
            format!("!{id}").blue(),
        ));
        return Ok("".into());
    }

    println!(
        "[mipsy] enter commands seperated by newlines\n\
              [mipsy] to run whenever breakpoint {} is hit",
        format!("!{id}").blue()
    );
    println!(
        "[mipsy] use an empty line or the command {} to finish",
        "end".bold().yellow()
    );

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

    prompt::success_nl(format!(
        "commands attached to breakpoint {}",
        format!("!{id}").blue()
    ));
    Ok("".into())
}

fn list_commands<K, V: Point>(points: &mut HashMap<K, V>, id: u32) -> CommandResult<String> {
    println!(
        "[mipsy] commands for breakpoint {}:",
        format!("!{id}").blue()
    );

    if let Some(br) = points.iter_mut().find(|bp| bp.1.get_id() == id) {
        let commands = br.1.get_commands();
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
    if !command_name.is_empty() {
        help.push(' ')
    };

    CommandError::WithTip {
        error: Box::new(error),
        tip: format!("try `{}{}`", help.bold(), command_name.bold()),
    }
}
