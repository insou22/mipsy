use std::sync::atomic::Ordering;

use crate::interactive::error::CommandError;
use crate::prompt;

use super::Command;
use super::*;
use colored::*;
use mipsy_lib::Register;

pub(super) fn args_numbers(args: &[ArgumentKind]) -> Vec<i64> {
    args.iter().cloned().map(i64::from).collect()
}

fn call_subcmds(
    cmd: &Command,
    helper: &mut MyHelper,
    args: &[ArgumentKind],
) -> CommandResult<String> {
    if let Some(arg) = args.get(0) {
        if let Some(cmd) = cmd
            .subcommands
            .iter()
            .find(|c| c.names.contains(&arg.clone().into()))
        {
            return (cmd._internal_exec)(cmd, helper, &args[1..]);
        }
    }

    step_syscall(helper)
}

pub(crate) fn command() -> Command {
    let times_arg = Argument::new(
        "times",
        |_, a, _| match a.parse::<i32>() {
            Ok(i) => Ok(ArgumentKind::Number(i as _)),
            Err(_) => Err(CommandError::WithTip {
                error: Box::new(CommandError::ArgExpectedI32 {
                    arg: "[times]".bright_magenta().to_string(),
                    instead: a.to_owned(),
                }),
                tip: format!("try `{} {}`", "help".bold(), "step".bold()),
            }),
        },
        |_, _, _| vec![],
    );

    Command::new()
        .with_name("step")
        .with_desc("step forwards or execute a subcommand")
        .with_name("s")
        .with_name("back")
        .with_optional_arg(times_arg.clone())
        .with_optional_arg(Argument::subcommands())
        .with_subcommand(
            Command::new()
            .with_name("back")
            .with_name("b")
            .with_optional_arg(times_arg)
            .with_help(format!(
                    "Steps backwards one instruction, or {0} instructions if specified.\n\
                    It will then print out which instruction will be executed next --\n\
                    \x20 i.e. using `{1}` will immediately execute said printed instruction.\n\
                    To step fowards (i.e. normal stepping), use `{1}`.",
                    "[times]".magenta(),
                    "step".bold(),
            ))
            .with_exec(|_, helper, args| step_back(helper, &args_numbers(args))),
        )
        .with_help(get_long_help())
        .with_exec(call_subcmds)
        .with_subcommand(
            Command::new()
            .with_name("syscall")
            .with_name("s")
            .with_name("sys")
            .with_help(subcmd_help())
            .with_subcommand(
                Command::new()
                .with_name("input")
                .with_name("in")
                .with_help(get_step_help_text(
                        "Steps forwards until your program asks for its next input, or finishes.",
                ))
                .with_exec(|_, helper, _| step_input(helper)),
            )
            .with_subcommand(
                Command::new()
                .with_name("output")
                .with_name("out")
                .with_name(get_step_help_text(
                        "Steps forwards until your program asks for its next output, or finishes.",
                ))
                .with_exec(|_, helper, _| step_output(helper)),
            )
            .with_subcommand(
                Command::new()
                .with_name("integer")
                .with_name("int")
                .with_help(get_step_help_text(
                        "Steps forwards until your program executes a syscall that requires\n\
                        reading or writing an integer, or finishes.",
                ))
                .with_exec(|_, helper, _| step_integer(helper)),
            )
            .with_subcommand(
                Command::new()
                .with_name("float")
                .with_help(get_step_help_text(
                        "Steps forwards until your program executes a syscall that requires\n\
                        reading or writing a float, or finishes.",
                ))
                .with_exec(|_, helper, _| step_float(helper)),
            )
            .with_subcommand(
                Command::new()
                .with_name("double")
                .with_help(get_step_help_text(
                        "Steps forwards until your program executes a syscall that requires\n\
                        reading or writing a double, or finishes.",
                ))
                .with_exec(|_, helper, _| step_double(helper)),
            )
            .with_subcommand(
                Command::new()
                .with_name("string")
                .with_name("str")
                .with_help(get_step_help_text(
                        "Steps forwards until your program executes a syscall that requires\n\
                        reading or writing a string, or finishes.",
                ))
                .with_exec(|_, helper, _| step_string(helper)),
            )
            .with_subcommand(
                Command::new()
                .with_name("character")
                .with_name("char")
                .with_help(get_step_help_text(
                        "Steps forwards until your program executes a syscall that requires\n\
                        reading or writing a character, or finishes.",
                ))
                .with_exec(|_, helper, _| step_character(helper)),
            )
            .with_subcommand(
                Command::new()
                .with_name("file")
                .with_help(get_step_help_text(
                        "Steps forwards until your program executes a syscall that opens,\n\
                        reads from, writes to, or closes a file, or finishes.",
                ))
                .with_exec(|_, helper, _| step_file(helper)),
            )
            .with_required_arg(Argument::subcommands())
            .with_exec(call_subcmds)
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

fn step_forward(helper: &mut MyHelper, args: &[i64]) -> Result<String, CommandError> {
    let times = args
        .first()
        .and_then(|&n| Some(i64::from(n)))
        .or(Some(1))
        .unwrap();
    if times.is_negative() {
        return step_back(
            helper,
            [times]
                .into_iter()
                .chain(args.iter().skip(1).cloned())
                .collect::<Vec<_>>()
                .as_ref(),
        );
    }

    if helper.state.exited {
        return Err(CommandError::ProgramExited);
    }

    helper.state.interrupted.store(false, Ordering::SeqCst);
    for _ in 0..times {
        let binary = helper
            .state
            .binary
            .as_ref()
            .ok_or(CommandError::MustLoadFile)?;
        let runtime = &helper.state.runtime;

        if let Ok(inst) = runtime.next_inst() {
            util::print_inst(
                &helper.state.iset,
                binary,
                inst,
                runtime.timeline().state().pc(),
                helper.state.program.as_deref(),
            );
        }

        let step = State::step(helper, true)?;

        if step | helper.state.interrupted.load(Ordering::SeqCst) {
            break;
        }
    }

    Ok("".into())
}

fn step_back(helper: &mut MyHelper, args: &[i64]) -> CommandResult<String> {
    let times = *args.first().or(Some(&1)).unwrap();
    if times.is_negative() {
        return step_forward(
            helper,
            [times.abs()]
                .into_iter()
                .chain(args.iter().skip(1).cloned())
                .collect::<Vec<_>>()
                .as_ref(),
        );
    }

    let mut backs = 0;
    let mut ran_out_of_history = false;
    helper.state.interrupted.store(false, Ordering::SeqCst);
    for _ in 0..times {
        let runtime = &mut helper.state.runtime;

        if runtime.timeline().timeline_len() == 2 && runtime.timeline().lost_history() {
            if backs == 0 {
                return Err(CommandError::RanOutOfHistory);
            }

            ran_out_of_history = true;
            break;
        }

        if runtime.timeline_mut().pop_last_state() {
            backs += 1;
            helper.state.exited = false;
        } else if backs == 0 {
            return Err(CommandError::CannotStepFurtherBack);
        }

        if helper.state.interrupted.load(Ordering::SeqCst) {
            break;
        }
    }

    let binary = helper
        .state
        .binary
        .as_ref()
        .ok_or(CommandError::MustLoadFile)?;
    let runtime = &helper.state.runtime;

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
            &helper.state.iset,
            binary,
            inst,
            runtime.timeline().state().pc(),
            helper.state.program.as_deref(),
        );
    }
    println!();

    Ok("".into())
}

fn step_syscall(helper: &mut MyHelper) -> Result<String, CommandError> {
    step_till_condition(helper, |_| true)
}

fn step_input(helper: &mut MyHelper) -> Result<String, CommandError> {
    step_till_condition(helper, |syscall| matches!(syscall, 5 | 6 | 7 | 8 | 12))
}

fn step_output(helper: &mut MyHelper) -> Result<String, CommandError> {
    step_till_condition(helper, |syscall| matches!(syscall, 1 | 2 | 3 | 4 | 11))
}

fn step_integer(helper: &mut MyHelper) -> Result<String, CommandError> {
    step_till_condition(helper, |syscall| matches!(syscall, 1 | 5))
}

fn step_float(helper: &mut MyHelper) -> Result<String, CommandError> {
    step_till_condition(helper, |syscall| matches!(syscall, 2 | 6))
}

fn step_double(helper: &mut MyHelper) -> Result<String, CommandError> {
    step_till_condition(helper, |syscall| matches!(syscall, 3 | 7))
}

fn step_string(helper: &mut MyHelper) -> Result<String, CommandError> {
    step_till_condition(helper, |syscall| matches!(syscall, 4 | 8))
}

fn step_character(helper: &mut MyHelper) -> Result<String, CommandError> {
    step_till_condition(helper, |syscall| matches!(syscall, 11 | 12))
}

fn step_file(helper: &mut MyHelper) -> Result<String, CommandError> {
    step_till_condition(helper, |syscall| matches!(syscall, 13 | 14 | 15 | 16))
}

fn step_till_condition<F>(helper: &mut MyHelper, condition: F) -> Result<String, CommandError>
where
    F: Fn(i32) -> bool,
{
    if helper.state.exited {
        return Err(CommandError::ProgramExited);
    }

    helper.state.interrupted.store(false, Ordering::SeqCst);
    while !helper.state.interrupted.load(Ordering::SeqCst) {
        let binary = helper
            .state
            .binary
            .as_ref()
            .ok_or(CommandError::MustLoadFile)?;
        let runtime = &helper.state.runtime;

        let stop = if let Ok(inst) = runtime.next_inst() {
            util::print_inst(
                &helper.state.iset,
                binary,
                inst,
                runtime.timeline().state().pc(),
                helper.state.program.as_deref(),
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

        let step = State::step(helper, true)?;

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

fn subcmd_help() -> String {
    get_step_help_text(
        format!(
            "\
                Available {1}s are:\n\
                \n\
                {0} {2}\t\t: syscalls 5, 6, 7, 8, 12\n\
                {0} {3}\t\t: syscalls 1, 2, 3, 4, 11\n\
                {0} {4}\t\t: syscalls 1, 5\n\
                {0} {5}\t\t: syscalls 2, 6\n\
                {0} {6}\t\t: syscalls 3, 7\n\
                {0} {7}\t\t: syscalls 4, 8\n\
                {0} {8}\t\t: syscalls 11, 12\n\
                {0} {9}\t\t: syscalls 13, 14, 15, 16\n\
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
    )
}
