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
    state: &mut State,
    helper: &MyHelper,
    label: &str,
    args: &[ArgumentKind],
) -> CommandResult<String> {
    match args.get(0) {
        Some(ArgumentKind::String(arg)) => {
            if let Some(cmd) = cmd.subcommands.iter().find(|c| c.names.contains(arg)) {
                return (cmd._internal_exec)(cmd, state, helper, label, &args[1..]);
            }
        }
        None | Some(ArgumentKind::Number(_)) => {}
    }

    step_syscall(state, label, helper)
}

fn subcmd() -> Command {
    Command::new()
        .with_name("syscall")
        .with_name("s")
        .with_name("sys")
        .with_subcommand(
            Command::new()
                .with_name("input")
                .with_name("in")
                .with_exec(|_, state, helper, label, _| step_input(state, label, helper)),
        )
        .with_subcommand(
            Command::new()
                .with_name("output")
                .with_name("out")
                .with_exec(|_, state, helper, label, _| step_output(state, label, helper)),
        )
        .with_subcommand(
            Command::new()
                .with_name("integer")
                .with_name("int")
                .with_exec(|_, state, helper, label, _| step_integer(state, label, helper)),
        )
        .with_subcommand(
            Command::new()
                .with_name("float")
                .with_exec(|_, state, helper, label, _| step_float(state, label, helper)),
        )
        .with_subcommand(
            Command::new()
                .with_name("double")
                .with_exec(|_, state, helper, label, _| step_double(state, label, helper)),
        )
        .with_subcommand(
            Command::new()
                .with_name("string")
                .with_name("str")
                .with_exec(|_, state, helper, label, _| step_string(state, label, helper)),
        )
        .with_subcommand(
            Command::new()
                .with_name("character")
                .with_name("char")
                .with_exec(|_, state, helper, label, _| step_character(state, label, helper)),
        )
        .with_subcommand(
            Command::new()
                .with_name("file")
                .with_exec(|_, state, helper, label, _| step_file(state, label, helper)),
        )
        .with_exec(call_subcmds)
}

pub(crate) fn command() -> Command {
    let times_arg = Argument::new(
        "times",
        |a, _| match a.parse::<i32>() {
            Ok(i) => Ok(ArgumentKind::Number(i as _)),
            Err(_) => Err(CommandError::WithTip {
                error: Box::new(CommandError::ArgExpectedI32 {
                    arg: "[times]".bright_magenta().to_string(),
                    instead: a.to_owned(),
                }),
                // tip: format!("try `{} {}`", "help".bold(), label.bold()),
                tip: format!("try TODOTODOIJJDSKJAKDJTODOOOOOOOOOOOOOTODOOOOOOOOOOOO"),
            }),
        },
        |_, _| vec![],
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
                .with_exec(|_, state, helper, label, args| {
                    step_back(state, label, &args_numbers(args), helper)
                }),
        )
        .with_subcommand(subcmd())
        .with_exec(call_subcmds)
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

fn step_forward(
    state: &mut State,
    label: &str,
    args: &[i64],
    helper: &MyHelper,
) -> Result<String, CommandError> {
    let times = args
        .first()
        .and_then(|&n| Some(i64::from(n)))
        .or(Some(1))
        .unwrap();
    if times.is_negative() {
        return step_back(
            state,
            label,
            [times]
                .into_iter()
                .chain(args.iter().skip(1).cloned())
                .collect::<Vec<_>>()
                .as_ref(),
            helper,
        );
    }

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

        let step = state.step(true, helper)?;

        if step | state.interrupted.load(Ordering::SeqCst) {
            break;
        }
    }

    Ok("".into())
}

fn step_back(
    state: &mut State,
    label: &str,
    args: &[i64],
    helper: &MyHelper,
) -> CommandResult<String> {
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

    let times = *args.first().or(Some(&1)).unwrap();
    if times.is_negative() {
        return step_forward(
            state,
            label,
            [times.abs()]
                .into_iter()
                .chain(args.iter().skip(1).cloned())
                .collect::<Vec<_>>()
                .as_ref(),
            helper,
        );
    }

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

fn step_syscall(state: &mut State, label: &str, helper: &MyHelper) -> Result<String, CommandError> {
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

    step_till_condition(state, |_| true, helper)
}

fn step_input(state: &mut State, label: &str, helper: &MyHelper) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program asks for its next input, or finishes.",
        ));
    }

    step_till_condition(
        state,
        |syscall| matches!(syscall, 5 | 6 | 7 | 8 | 12),
        helper,
    )
}

fn step_output(state: &mut State, label: &str, helper: &MyHelper) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program asks for its next output, or finishes.",
        ));
    }

    step_till_condition(
        state,
        |syscall| matches!(syscall, 1 | 2 | 3 | 4 | 11),
        helper,
    )
}

fn step_integer(state: &mut State, label: &str, helper: &MyHelper) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program executes a syscall that requires\n\
                 reading or writing an integer, or finishes.",
        ));
    }

    step_till_condition(state, |syscall| matches!(syscall, 1 | 5), helper)
}

fn step_float(state: &mut State, label: &str, helper: &MyHelper) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program executes a syscall that requires\n\
                 reading or writing a float, or finishes.",
        ));
    }

    step_till_condition(state, |syscall| matches!(syscall, 2 | 6), helper)
}

fn step_double(state: &mut State, label: &str, helper: &MyHelper) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program executes a syscall that requires\n\
                 reading or writing a double, or finishes.",
        ));
    }

    step_till_condition(state, |syscall| matches!(syscall, 3 | 7), helper)
}

fn step_string(state: &mut State, label: &str, helper: &MyHelper) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program executes a syscall that requires\n\
                 reading or writing a string, or finishes.",
        ));
    }

    step_till_condition(state, |syscall| matches!(syscall, 4 | 8), helper)
}

fn step_character(
    state: &mut State,
    label: &str,
    helper: &MyHelper,
) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program executes a syscall that requires\n\
                 reading or writing a character, or finishes.",
        ));
    }

    step_till_condition(state, |syscall| matches!(syscall, 11 | 12), helper)
}

fn step_file(state: &mut State, label: &str, helper: &MyHelper) -> Result<String, CommandError> {
    if label == "__help__" {
        return Ok(get_step_help_text(
            "Steps forwards until your program executes a syscall that opens,\n\
                 reads from, writes to, or closes a file, or finishes.",
        ));
    }

    step_till_condition(
        state,
        |syscall| matches!(syscall, 13 | 14 | 15 | 16),
        helper,
    )
}

fn step_till_condition<F>(
    state: &mut State,
    condition: F,
    helper: &MyHelper,
) -> Result<String, CommandError>
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

        let step = state.step(true, helper)?;

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
