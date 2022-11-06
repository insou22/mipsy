use std::sync::atomic::Ordering;
use std::vec;

use crate::interactive::error::CommandError;
use crate::prompt;

use super::*;
use super::Command;
use colored::*;

pub(crate) fn step_command() -> Command {
    let subcommands = vec![command(
        "back",
        vec!["b"],
        vec![],
        vec!["times"],
        vec![],
        "",
        |_, state, label, args| step_back(state, label, args),
    )];

    // TODO:
    //  - back alias
    command(
        "step",
        vec!["s"],
        vec![],
        vec!["times"],
        subcommands,
        &format!("step forwards one (or {}) instruction", "[times]".magenta()),
        |cmd, state, label, args| {
            if label == "__help__" && args.is_empty() {
                return Ok(get_long_help());
            }

            let cmd = if args.is_empty() {
                None
            } else {
                cmd.subcommands
                    .iter()
                    .find(|c| c.name == args[0] || c.aliases.contains(&args[0]))
            };
            match cmd {
                Some(cmd) => cmd.exec(state, label, &args[1..]),
                None => step_forward(state, label, args),
            }
        },
    )
}

fn get_long_help() -> String {
    format!("TODO: long help")
}

fn step_forward(state: &mut State, label: &str, args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(format!(
            "Steps forwards one instruction, or {} instructions if specified.\n\
                 This will run in \"verbose\" mode, printing out the instruction that was\n\
             \x20 executed, and verbosely printing any system calls that are executed.\n\
                 To step backwards (i.e. back in time), use `{}`.",
            "[times]".magenta(),
            "back".bold(),
        ));
    }

    let times = match args.first() {
        Some(arg) => match arg.parse::<i32>() {
            Ok(num) => {
                if num.is_negative() {
                    // TODO: rest of args?
                    return step_back(state, label, &[num.abs().to_string()]);
                }

                Ok(num)
            }
            Err(_) => Err(CommandError::WithTip {
                error: Box::new(CommandError::ArgExpectedI32 {
                    arg: "[times]".bright_magenta().to_string(),
                    instead: arg.to_owned(),
                }),
                tip: format!("try `{} {}`", "help".bold(), label.bold()),
            }),
        },
        None => Ok(1),
    }?;

    if state.exited {
        return Err(CommandError::ProgramExited);
    }

    state.interrupted.store(false, Ordering::SeqCst);
    for _ in 0..times {
        let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
        let runtime = &state.runtime;

        if let Ok(inst) = runtime.next_inst() {
            util::print_inst(
                &state.iset,
                binary,
                inst,
                runtime.timeline().state().pc(),
                state.program.as_deref(),
            );
        }

        let step = state.step(true)?;

        if step | state.interrupted.load(Ordering::SeqCst) {
            break;
        }
    }

    Ok("".into())
}

fn step_back(state: &mut State, label: &str, args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(format!(
            "Steps backwards one instruction, or {0} instructions if specified.\n\
                 It will then print out which instruction will be executed next --\n\
             \x20 i.e. using `{1}` will immediately execute said printed instruction.\n\
                 To step fowards (i.e. normal stepping), use `{1}`.",
            "[times]".magenta(),
            "step".bold(),
        ));
    }

    let times = match args.first() {
        Some(arg) => match arg.parse::<i32>() {
            Ok(num) => {
                if num.is_negative() {
                    // TODO: rest of args?
                    return step_forward(state, label, &[num.abs().to_string()]);
                }

                Ok(num)
            }
            Err(_) => Err(CommandError::WithTip {
                error: Box::new(CommandError::ArgExpectedI32 {
                    arg: "[times]".bright_magenta().to_string(),
                    instead: arg.to_owned(),
                }),
                tip: format!("try `{} {}`", "help".bold(), label.bold()),
            }),
        },
        None => Ok(1),
    }?;

    let mut backs = 0;
    let mut ran_out_of_history = false;
    for _ in 0..times {
        let runtime = &mut state.runtime;

        if runtime.timeline().timeline_len() == 2 && runtime.timeline().lost_history() {
            if backs == 0 {
                return Err(CommandError::RanOutOfHistory);
            }

            ran_out_of_history = true;
            break;
        }

        if runtime.timeline_mut().pop_last_state() {
            backs += 1;
            state.exited = false;
        } else if backs == 0 {
            return Err(CommandError::CannotStepFurtherBack);
        }
    }

    let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
    let runtime = &state.runtime;

    let pluralise = if backs != 1 { "s" } else { "" };

    let mut text = format!(
        "stepped back {} instruction{}",
        backs.to_string().magenta(),
        pluralise
    );

    if ran_out_of_history {
        text.push_str(" (before running out of history)");
    } else if backs < times {
        text.push_str(" (reached start of program)");
    }
    text.push_str(", next instruction will be:");

    prompt::success(text);
    if let Ok(inst) = runtime.next_inst() {
        util::print_inst(
            &state.iset,
            binary,
            inst,
            runtime.timeline().state().pc(),
            state.program.as_deref(),
        );
    }
    println!();

    Ok("".into())
}
