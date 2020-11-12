use crate::interactive::{error::CommandError, prompt};

use super::*;
use colored::*;

pub(crate) fn help_command() -> Command {
    command(
        "help",
        vec!["h", "?"],
        vec![],
        vec!["command"],
        "print this help text, or specific help for a command",
        &format!(
            "Prints the general help text for all mipsy commands, or more in-depth\n\
         \x20 help for a specific {} if specified, including available aliases.",
             "[command]".magenta()
        ),
        |state, _label, args| {
            if let Some(command) = args.first() {
                let command = state.commands.iter()
                        .find(|cmd| &cmd.name == command || cmd.aliases.contains(command))
                        .ok_or(CommandError::HelpUnknownCommand { command: command.clone() })?;

                println!("\n{}\n", get_command_formatted(command));
                println!("{}", command.long_description);
                if !command.aliases.is_empty() {
                    prompt::banner("\naliases".green().bold());
                    println!("{}", command.aliases.iter().map(|s| s.yellow().bold().to_string()).collect::<Vec<String>>().join(", "));
                }
                println!();

                return Ok(())
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
                    Arguments::VarArgs { required } => {
                        len += required.len();
                        for arg in required.iter() {
                            len += arg.len() + 2;
                        }

                        len += " {args}".len();
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
                        Arguments::VarArgs { required } => {
                            "".magenta()       .to_string().len() * required.len() +
                            "".bright_magenta().to_string().len()
                        }
                    };

                let name_args = get_command_formatted(command);

                let char_len = name_args.len() - extra_color_len;
                let extra_padding = max_len - char_len;

                println!("{}{} - {}", name_args, " ".repeat(extra_padding), command.description);
            }

            println!();

            Ok(())
        }
    )
}

fn get_command_formatted(cmd: &Command) -> String {
    let mut parts = vec![
        cmd.name.yellow().bold().to_string(),
    ];

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
        Arguments::VarArgs { required } => {
            parts.append(
                &mut required.iter()
                        .map(|arg| format!("<{}>", arg).magenta().to_string())
                        .collect::<Vec<String>>()
            );

            parts.push("{args}".magenta().to_string());
        }
    }

    parts.join(" ")
}
