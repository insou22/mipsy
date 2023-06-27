use std::sync::atomic::Ordering;
use std::vec;

use crate::interactive::error::CommandError;
use crate::prompt;

use super::Command;
use super::*;
use colored::*;
use mipsy_lib::Register;

pub(crate) fn step_command() -> Command {
    let subcommands = vec![
        command(
            "back",
            vec!["b"],
            vec![],
            vec!["times"],
            vec![],
            "",
            |_, state, label, args| step_back(state, label, args),
        ),
        command(
            "syscall",
            vec!["s", "sys"],
            vec![],
            vec![],
            vec![
                command(
                    "input",
                    vec!["in"],
                    vec![],
                    vec![],
                    vec![],
                    "",
                    |_, state, label, args| step_input(state, label, args),
                ),
                command(
                    "output",
                    vec!["out"],
                    vec![],
                    vec![],
                    vec![],
                    "",
                    |_, state, label, args| step_output(state, label, args),
                ),
                command(
                    "integer",
                    vec!["int"],
                    vec![],
                    vec![],
                    vec![],
                    "",
                    |_, state, label, args| step_integer(state, label, args),
                ),
                command(
                    "float",
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                    "",
                    |_, state, label, args| step_float(state, label, args),
                ),
                command(
                    "double",
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                    "",
                    |_, state, label, args| step_double(state, label, args),
                ),
                command(
                    "string",
                    vec!["str"],
                    vec![],
                    vec![],
                    vec![],
                    "",
                    |_, state, label, args| step_string(state, label, args),
                ),
                command(
                    "character",
                    vec!["char"],
                    vec![],
                    vec![],
                    vec![],
                    "",
                    |_, state, label, args| step_character(state, label, args),
                ),
                command(
                    "file",
                    vec![],
                    vec![],
                    vec![],
                    vec![],
                    "",
                    |_, state, label, args| step_file(state, label, args),
                ),
            ],
            "",
            |cmd, state, label, args| {
                let cmd = args.get(0).and_then(|arg| {
                    cmd.subcommands
                        .iter()
                        .find(|c| &c.name == arg || c.aliases.contains(arg))
                });
                match cmd {
                    Some(cmd) => cmd.exec(state, label, &args[1..]),
                    None => step_syscall(state, label, args),
                }
            },
        ),
    ];

    command(
        "step",
        vec!["s", "back"],
        vec![],
        vec!["times", "subcommand"],
        subcommands,
        "step forwards or execute a subcommand",
        |cmd, state, label, args| {
            if label == "__help__" && args.is_empty() {
                return Ok(get_long_help());
            }

            if label == "back" {
                return step_back(state, label, args);
            }

            let cmd = args.get(0).and_then(|arg| {
                cmd.subcommands
                    .iter()
                    .find(|c| &c.name == arg || c.aliases.contains(arg))
            });
            match cmd {
                Some(cmd) => cmd.exec(state, label, &args[1..]),
                None => step_forward(state, label, args),
            }
        },
    )
}

fn get_long_help() -> String {
    format!(
        "A collection of commands for {3}ping through the program. Available {0}s are:\n\
         \n\
         {3} {4}    : steps backwards instead of forwards\n\
         {3} {5} : steps forwards until the next syscall\n\
         \n\
         {6} {7} will provide more information about the specified subcommand.\n\
         \n\
         By default, this steps forwards one instruction, or {1} instructions if specified.\n\
         This will run in \"verbose\" mode, printing out the instruction that was\n\
         \x20 executed, and verbosely printing any system calls that are executed.\n\
         To step backwards (i.e. back in time), use `{2}`.",
        "[subcommand]".magenta(),
        "[times]".magenta(),
        "back".bold(),
        "step".yellow().bold(),
        "back".purple(),
        "syscall".purple(),
        "help step".white().bold(),
        "[subcommand]".magenta().bold(),
    )
}

fn step_forward(state: &mut State, label: &str, args: &[String]) -> Result<String, CommandError> {
    let times = match args.first() {
        Some(arg) => match arg.parse::<i32>() {
            Ok(num) => {
                if num.is_negative() {
                    return step_back(
                        state,
                        label,
                        [num.abs().to_string()]
                            .into_iter()
                            .chain(args.iter().skip(1).cloned())
                            .collect::<Vec<String>>()
                            .as_ref(),
                    );
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
                    return step_forward(
                        state,
                        label,
                        [num.abs().to_string()]
                            .into_iter()
                            .chain(args.iter().skip(1).cloned())
                            .collect::<Vec<String>>()
                            .as_ref(),
                    );
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
    state.interrupted.store(false, Ordering::SeqCst);
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

        if state.interrupted.load(Ordering::SeqCst) {
            break;
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

fn step_syscall(state: &mut State, label: &str, _args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            format!(
                "\
                Available {1}s are:\n\
                \n\
                {0} {2}     : syscalls 5, 6, 7, 8, 12\n\
                {0} {3}    : syscalls 1, 2, 3, 4, 11\n\
                {0} {4}   : syscalls 1, 5\n\
                {0} {5}     : syscalls 2, 6\n\
                {0} {6}    : syscalls 3, 7\n\
                {0} {7}    : syscalls 4, 8\n\
                {0} {8} : syscalls 11, 12\n\
                {0} {9}      : syscalls 13, 14, 15, 16\n\
                \n\
                {10} {11} will provide more information about the specified subcommand.\n\
                \n\
                By default, this steps forwards until your program's next syscall, or finishes.",
                "step syscall".bold().yellow(),
                "[type]".magenta(),
                "input".purple(),
                "output".purple(),
                "integer".purple(),
                "float".purple(),
                "double".purple(),
                "string".purple(),
                "character".purple(),
                "file".purple(),
                "help step syscall".white().bold(),
                "[type]".magenta().bold(),
            )
            .as_ref(),
        ));
    }

    step_till_condition(state, |_| true)
}

fn step_input(state: &mut State, label: &str, _args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program asks for its next input, or finishes.",
        ));
    }

    step_till_condition(state, |syscall| matches!(syscall, 5 | 6 | 7 | 8 | 12))
}

fn step_output(state: &mut State, label: &str, _args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program asks for its next output, or finishes.",
        ));
    }

    step_till_condition(state, |syscall| matches!(syscall, 1 | 2 | 3 | 4 | 11))
}

fn step_integer(state: &mut State, label: &str, _args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program executes a syscall that requires\n\
                 reading or writing an integer, or finishes.",
        ));
    }

    step_till_condition(state, |syscall| matches!(syscall, 1 | 5))
}

fn step_float(state: &mut State, label: &str, _args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program executes a syscall that requires\n\
                 reading or writing a float, or finishes.",
        ));
    }

    step_till_condition(state, |syscall| matches!(syscall, 2 | 6))
}

fn step_double(state: &mut State, label: &str, _args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program executes a syscall that requires\n\
                 reading or writing a double, or finishes.",
        ));
    }

    step_till_condition(state, |syscall| matches!(syscall, 3 | 7))
}

fn step_string(state: &mut State, label: &str, _args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program executes a syscall that requires\n\
                 reading or writing a string, or finishes.",
        ));
    }

    step_till_condition(state, |syscall| matches!(syscall, 4 | 8))
}

fn step_character(
    state: &mut State,
    label: &str,
    _args: &[String],
) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program executes a syscall that requires\n\
                 reading or writing a character, or finishes.",
        ));
    }

    step_till_condition(state, |syscall| matches!(syscall, 11 | 12))
}

fn step_file(state: &mut State, label: &str, _args: &[String]) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program executes a syscall that opens,\n\
                 reads from, writes to, or closes a file, or finishes.",
        ));
    }

    step_till_condition(state, |syscall| matches!(syscall, 13 | 14 | 15 | 16))
}

fn step_till_condition<F>(state: &mut State, condition: F) -> Result<String, CommandError>
where
    F: Fn(i32) -> bool,
{
    if state.exited {
        return Err(CommandError::ProgramExited);
    }

    state.interrupted.store(false, Ordering::SeqCst);
    while !state.interrupted.load(Ordering::SeqCst) {
        let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
        let runtime = &state.runtime;

        let stop = if let Ok(inst) = runtime.next_inst() {
            util::print_inst(
                &state.iset,
                binary,
                inst,
                runtime.timeline().state().pc(),
                state.program.as_deref(),
            );

            if inst == 0xC {
                let syscall = runtime
                    .timeline()
                    .state()
                    .read_register(Register::V0.to_u32())
                    .unwrap_or(-1);
                condition(syscall)
            } else {
                false
            }
        } else {
            false
        };

        let step = state.step(true)?;

        if step || stop {
            break;
        }
    }

    Ok("".into())
}

fn get_step_help_text(unique_text: &str) -> String {
    format!(
        "{}\n\
         This will run in \"verbose\" mode, printing out each instruction that was\n\
     \x20 executed, and verbosely printing any system calls that are executed.\n\
         To step backwards (i.e. back in time), use `{}`.",
        unique_text,
        "step back".bold(),
    )
}
