use crate::{
    interactive::{commands::watchpoint::args_text, error::CommandError},
    prompt,
};

use super::*;
use colored::*;

pub(crate) fn command() -> Command {
    Command::new()
        .with_name("help")
        .with_name("h")
        .with_name("?")
        .with_desc("print this help text, or specific help for a command")
        .with_exact_args()
        // TODO: context for sanitisation
        .with_optional_arg(Argument::new(
            "command",
            |_, a, _| Ok(ArgumentKind::String(a.to_owned())),
            |_, _, h| {
                h.state
                    .commands
                    .iter()
                    .map(|c| c.name().to_owned())
                    .collect()
            },
        ))
        .with_help(format!(
            "Prints the general help text for all mipsy commands, or more in-depth\n\
                \x20 help for a specific {} if specified, including available aliases.",
            "[command]".magenta()
        ))
        .with_exec(|_, helper, args| {
            let args = &args_text(args);
            if let Some(command) = args.first() {
                let mut command = &helper.state.find_command(&command).ok_or(
                    CommandError::HelpUnknownCommand {
                        command: command.to_owned(),
                    },
                )?;

                let mut args = &args[1..];
                let mut parts = vec![command
                    .names
                    .get(0)
                    .expect("no command name")
                    .yellow()
                    .bold()
                    .to_string()];

                while !args.is_empty() {
                    let subcmd = command
                        .subcommands
                        .iter()
                        .find(|c| c.names.contains(&args[0]));
                    if let Some(subcmd) = subcmd {
                        command = subcmd;
                        parts.push(
                            subcmd
                                .names
                                .get(0)
                                .expect("command has no name")
                                .yellow()
                                .bold()
                                .to_string(),
                        );
                    }

                    args = &args[1..];
                }

                println!("\n{}\n", get_command_formatted(command, parts));
                println!("{}", command.help);

                if !command.names[1..].is_empty() {
                    prompt::banner("\naliases".green().bold());
                    println!(
                        "{}",
                        command.names[1..]
                            .iter()
                            .map(|s| s.yellow().bold().to_string())
                            .collect::<Vec<String>>()
                            .join(", ")
                    );
                }
                println!();
                return Ok("".into());
            }

            let mut max_len = 0;

            for command in helper.state.commands.iter() {
                let mut len = command.names.get(0).expect("no named command").len();

                match &command.args {
                    Arguments::Exactly { required, optional } => {
                        len += required.len();
                        for arg in required.iter() {
                            len += arg.name.len() + 2;
                        }

                        len += optional.len();
                        for arg in optional.iter() {
                            len += arg.name.len() + 2;
                        }
                    }
                    Arguments::VarArgs { required, format } => {
                        len += required.len();
                        for arg in required.iter() {
                            len += arg.name.len() + 2;
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
            for command in helper.state.commands.iter() {
                let extra_color_len = "".yellow().bold().to_string().len()
                    + match &command.args {
                        Arguments::Exactly { required, optional } => {
                            "".magenta().to_string().len() * required.len()
                                + "".bright_magenta().to_string().len() * optional.len()
                        }
                        Arguments::VarArgs {
                            required,
                            format: _,
                        } => {
                            "".magenta().to_string().len() * required.len()
                                + "".bright_magenta().to_string().len()
                        }
                    };

                let parts = vec![command
                    .names
                    .get(0)
                    .expect("command has no name")
                    .yellow()
                    .bold()
                    .to_string()];
                let name_args = get_command_formatted(command, parts);

                let char_len = name_args.len() - extra_color_len;
                let extra_padding = max_len - char_len;

                println!(
                    "{}{} - {}",
                    name_args,
                    " ".repeat(extra_padding),
                    command.description
                );
            }
            println!(
                "{}{} - repeat the previous command",
                "<enter>".yellow().bold(),
                " ".repeat(max_len - 7)
            );

            println!();

            Ok("".into())
        })
}

fn get_command_formatted(cmd: &Command, mut parts: Vec<String>) -> String {
    match &cmd.args {
        Arguments::Exactly { required, optional } => {
            parts.append(
                &mut required
                    .iter()
                    .map(|arg| format!("<{}>", arg.name).magenta().to_string())
                    .collect::<Vec<String>>(),
            );

            parts.append(
                &mut optional
                    .iter()
                    .map(|arg| format!("[{}]", arg.name).bright_magenta().to_string())
                    .collect::<Vec<String>>(),
            );
        }
        Arguments::VarArgs { required, format } => {
            parts.append(
                &mut required
                    .iter()
                    .map(|arg| format!("<{}>", arg.name).magenta().to_string())
                    .collect::<Vec<String>>(),
            );

            parts.push(format.to_string());
        }
    }

    parts.join(" ")
}
