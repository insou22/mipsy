use super::State;

use colored::*;
use super::prompt;
use super::error::CommandResult;
use super::error::CommandError;

pub(crate) struct Command {
    pub(crate) name: String,
    pub(crate) aliases: Vec<String>,
    pub(crate) required_args: Vec<String>,
    pub(crate) optional_args: Vec<String>,
    pub(crate) description: String,
    pub(crate) exec: fn(&mut State, &str, &[String]) -> CommandResult<()>,
}

pub(crate) fn load_command() -> Command {
    command(
        "load",
        vec!["l"],
        vec!["file"],
        vec![],
        "load a MIPS file to run",
        |state, _label, args| {
            let path = &args[0];

            let program = std::fs::read_to_string(path)
                .map_err(|err| CommandError::CannotReadFile { path: path.clone(), os_error: err.to_string() })?;
            
            let binary = mipsy_lib::compile(&state.iset, &program)
                .map_err(|err| CommandError::CannotCompile { path: path.clone(), program: program.clone(), mipsy_error: err })?;

            let runtime = mipsy_lib::run(&binary)
                .map_err(|err| CommandError::CannotCompile { path: path.clone(), program: program.clone(), mipsy_error: err })?;

            state.binary  = Some(binary);
            state.runtime = Some(runtime);
            state.exited  = false;
            prompt::success_nl("file loaded");

            Ok(())
        }
    )
}

pub(crate) fn step_command() -> Command {
    command(
        "step",
        vec!["s"],
        vec![],
        vec!["times"],
        &format!("step forwards one (or {}) instruction", "[times]".magenta()),
        |state, _label, args| {
            let times = match args.first() {
                Some(arg) => expect_u32(
                    "step",
                    &"[times]".bright_magenta().to_string(),
                    arg, 
                    Some(|neg| 
                        format!("try `{}{}`", "back ".bold(), (-1 * neg as i32).to_string().bold())
                    )
                ),
                None => Ok(1),
            }?;

            if state.exited {
                return Err(CommandError::ProgramExited);
            }

            for _ in 0..times {
                let binary  = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
                let runtime = state.runtime.as_ref().ok_or(CommandError::MustLoadFile)?;

                if let Ok(inst) = runtime.next_inst() {
                    state.print_inst(binary, inst, runtime.state().get_pc());
                }

                state.step(true)?;
                if state.exited {
                    break;
                }
            }

            Ok(())
        }
    )
}

pub(crate) fn back_command() -> Command {
    command(
        "back",
        vec!["b"],
        vec![],
        vec!["times"],
        &format!("step backwards one (or {}) instruction", "[times]".magenta()),
        |state, _label, args| {
            let times = match args.first() {
                Some(arg) => expect_u32(
                    "back",
                    &"[times]".bright_magenta().to_string(),
                    arg, 
                    Some(|neg| 
                        format!("try `{}{}`", "step ".bold(), (-1 * neg as i32).to_string().bold())
                    )
                ),
                None => Ok(1),
            }?;

            let mut backs = 0;
            for _ in 0..times {
                let runtime = state.runtime.as_mut().ok_or(CommandError::MustLoadFile)?;

                if runtime.back() {
                    backs += 1;
                    state.exited = false;
                } else {
                    if backs == 0 {
                        return Err(CommandError::CannotStepFurtherBack);
                    }
                }
            }

            let binary  = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
            let runtime = state.runtime.as_ref().ok_or(CommandError::MustLoadFile)?;

            let pluralise = if backs != 1 { "s" } else { "" };

            let mut text = String::from(format!("stepped back {} instruction{}", backs.to_string().magenta(), pluralise));
            if backs < times {
                text.push_str(" (reached start of program)");
            }
            text.push_str(", next instruction will be:");

            prompt::success(text);
            if let Ok(inst) = runtime.next_inst() {
                state.print_inst(binary, inst, runtime.state().get_pc());
            }
            println!();

            Ok(())
        }
    )
}

pub(crate) fn run_command() -> Command {
    command(
        "run",
        vec!["r"],
        vec![],
        vec![],
        "run the currently loaded program until it finishes",
        |state, _label, _args| {
            state.run()
        }
    )
}

pub(crate) fn reset_command() -> Command {
    command(
        "reset",
        vec!["re"],
        vec![],
        vec![],
        "reset the currently loaded program to its initial state",
        |state, _label, _args| {
            state.reset()?;
            prompt::success_nl("program reset");

            Ok(())
        }
    )
}

pub(crate) fn help_command() -> Command {
    command(
        "help",
        vec!["h", "?"],
        vec![],
        vec!["command"],
        "print this help text, or specific help for a command",
        |state, _label, _args| {
            let mut max_len = 0;

            for command in state.commands.iter() {
                let mut len = command.name.len();

                len += command.required_args.len();
                for arg in command.required_args.iter() {
                    len += arg.len() + 2;
                }

                len += command.optional_args.len();
                for arg in command.optional_args.iter() {
                    len += arg.len() + 2;
                }

                if len > max_len {
                    max_len = len;
                }
            }

            println!("{}", "\nCOMMANDS:".green().bold());
            for command in state.commands.iter() {
                let extra_color_len = 
                    "".yellow().bold() .to_string().len() +
                    "".magenta()       .to_string().len() * command.required_args.len() +
                    "".bright_magenta().to_string().len() * command.optional_args.len();

                let mut parts = vec![
                    command.name.yellow().bold().to_string(),
                ];

                parts.append(
                    &mut command.required_args.iter()
                            .map(|arg| format!("<{}>", arg).magenta().to_string())
                            .collect::<Vec<String>>()
                );

                parts.append(
                    &mut command.optional_args.iter()
                        .map(|arg| format!("[{}]", arg).bright_magenta().to_string())
                        .collect::<Vec<String>>()
                );

                let name_args = parts.join(" ");
                let char_len = name_args.len() - extra_color_len;
                let extra_padding = max_len - char_len;

                println!("{}{} - {}", name_args, " ".repeat(extra_padding), command.description);
            }

            println!();

            Ok(())
        }
    )
}

pub(crate) fn exit_command() -> Command {
    command(
        "exit",
        vec![],
        vec![],
        vec![],
        "exit mipsy",
        |_state, _label, _args| {
            std::process::exit(0);
        }
    )
}

fn command<S: Into<String>>(name: S, aliases: Vec<S>, required_args: Vec<S>, optional_args: Vec<S>, desc: S, exec: fn(&mut State, &str, &[String]) -> CommandResult<()>) -> Command {
    Command {
        name: name.into(),
        description: desc.into(),
        aliases: aliases.into_iter().map(S::into).collect(),
        required_args: required_args.into_iter().map(S::into).collect(),
        optional_args: optional_args.into_iter().map(S::into).collect(),
        exec,
    }
}

fn expect_u32<F>(command: &str, name: &str, arg: &str, neg_tip: Option<F>) -> CommandResult<u32>
where
    F: Fn(i32) -> String
{
    match arg.parse::<u32>() {
        Ok(num) => Ok(num),
        Err(_)  => Err({
            let err = CommandError::ArgExpectedU32 { arg: name.to_string(), instead: arg.to_string() };

            match (arg.parse::<i32>(), neg_tip) {
                (Ok(neg), Some(f)) => 
                    CommandError::WithTip { 
                        error: Box::new(err), 
                        tip: f(neg),
                    },
                _ => 
                    CommandError::WithTip {
                        error: Box::new(err),
                        tip: format!("try `{} {}`", "help".bold(), command.bold()),
                    },
            }
        }),
    }
}
