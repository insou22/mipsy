use std::sync::atomic::Ordering;

use crate::interactive::error::CommandError;

use super::*;
use super::Command;
use colored::*;

pub(crate) fn step2syscall_command() -> Command {
    command(
        "step2syscall",
        vec!["s2s"],
        vec![],
        vec![],
        vec![],
        "step forwards until next syscall",
        |state, label, _args| {
            if label == "__help__" {
                return Ok(
                    format!(
                        "Steps forwards until your program's next syscall.\n\
                         This will run in \"verbose\" mode, printing out each instruction that was\n\
                     \x20 executed, and verbosely printing any system calls that are executed.\n\
                         To step backwards (i.e. back in time), use `{}`.",
                        "back".bold(),
                    ),
                )
            }

            if state.exited {
                return Err(CommandError::ProgramExited);
            }

            state.interrupted.store(false, Ordering::SeqCst);
            while !state.interrupted.load(Ordering::SeqCst) {
                let binary  = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
                let runtime = &state.runtime;

                let syscall = if let Ok(inst) = runtime.next_inst() {
                    util::print_inst(&state.iset, binary, inst, runtime.timeline().state().pc(), state.program.as_deref());

                    inst == 0xC
                } else {
                    false
                };

                let step = state.step(true)?;

                if step || syscall {
                    break;
                }
                
            }

            Ok("".into())
        }
    )
}
