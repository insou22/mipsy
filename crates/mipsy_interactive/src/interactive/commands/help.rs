use crate::interactive::{error::CommandError, prompt};

use super::*;
use colored::*;

pub(crate) fn help_command() -> Command {
    command(
        "help",
        vec!["h", "?"],
        vec![],
        vec!["command"],
        vec![],
        "print this help text, or specific help for a command",
        |_, state, label, args| {
            if label == "__help__" {
                return Ok(
                    format!(
                        "Prints the general help text for all mipsy commands, or more in-depth\n\
                     \x20 help for a specific {} if specified, including available aliases.",
                         "[command]".magenta()
                    ),
                )
            }

            if let Some(command) = args.first() {
                let mut command = &state.find_command(command).ok_or(CommandError::HelpUnknownCommand { command: command.clone() })?;

                let args = &args[1..];
                let mut parts = vec![
                    command.name.yellow().bold().to_string(),
                ];

                // TODO(joshh): include optional/required args for subcommands?
                if !args.is_empty() {
                    let subcmd = command.subcommands.iter()
                            .find(|c| c.name == args[0] || c.aliases.contains(&args[0]));
                    if let Some(subcmd) = subcmd {
                        command = subcmd;
                        parts.push(command.name.yellow().bold().to_string());
                    }
                }

                println!("\n{}\n", get_command_formatted(command, parts));
                println!("{}", command.exec(state, "__help__", args).unwrap());

                if !command.aliases.is_empty() {
                    prompt::banner("\naliases".green().bold());
                    println!("{}", command.aliases.iter().map(|s| s.yellow().bold().to_string()).collect::<Vec<String>>().join(", "));
                }
                println!();
                return Ok("".into())
            }

            let mut max_len = 0;

            for command in state.commands.iter() {
                let mut len = command.name.len();

                match &command.args {
                    Arguments::Exactly { required, optional } => {
                        len += required.len();
                        for arg in required.iter() {
                            len += arg.len() + 2;
                        }
        
                        len += optional.len();
                        for arg in optional.iter() {
                            len += arg.len() + 2;
                        }
                    }
                    Arguments::VarArgs { required, format } => {
                        len += required.len();
                        for arg in required.iter() {
                            len += arg.len() + 2;
                        }

                        len += 1;

                        len += format.len();
                    }
                }

                if len > max_len {
                    max_len = len;
                }
            }

            println!("{}", "\nCOMMANDS:".green().bold());
            for command in state.commands.iter() {
                let extra_color_len = 
                    "".yellow().bold() .to_string().len() + 
                    match &command.args {
                        Arguments::Exactly { required, optional } => {
                            "".magenta()       .to_string().len() * required.len() +
                            "".bright_magenta().to_string().len() * optional.len()
                        }
                        Arguments::VarArgs { required, format: _ } => {
                            "".magenta()       .to_string().len() * required.len() +
                            "".bright_magenta().to_string().len()
                        }
                    };

                let parts = vec![
                    command.name.yellow().bold().to_string(),
                ];
                let name_args = get_command_formatted(command, parts);

                let char_len = name_args.len() - extra_color_len;
                let extra_padding = max_len - char_len;

                println!("{}{} - {}", name_args, " ".repeat(extra_padding), command.description);
            }
            println!("{}{} - repeat the previous command", "<enter>".yellow().bold(), " ".repeat(max_len - 7));

            println!();

            Ok("".into())
        }
    )
}

fn get_command_formatted(cmd: &Command, mut parts: Vec<String>) -> String {
    match &cmd.args {
        Arguments::Exactly { required, optional } => {
            parts.append(
                &mut required.iter()
                        .map(|arg| format!("<{}>", arg).magenta().to_string())
                        .collect::<Vec<String>>()
            );

            parts.append(
                &mut optional.iter()
                    .map(|arg| format!("[{}]", arg).bright_magenta().to_string())
                    .collect::<Vec<String>>()
            );
        }
        Arguments::VarArgs { required, format } => {
            parts.append(
                &mut required.iter()
                        .map(|arg| format!("<{}>", arg).magenta().to_string())
                        .collect::<Vec<String>>()
            );

            parts.push(format.to_string());
        }
    }

    parts.join(" ")
}
