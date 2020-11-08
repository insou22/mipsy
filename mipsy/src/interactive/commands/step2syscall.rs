use crate::interactive::error::CommandError;

use super::*;
use super::{Command};
use colored::*;

pub(crate) fn step2syscall_command() -> Command {
    command(
        "step2syscall",
        vec!["s2s"],
        vec![],
        vec![],
        "step forwards until next syscall",
        &format!(
            "Steps forwards until your program's next syscall.\n\
             This will run in \"verbose\" mode, printing out each instruction that was\n\
         \x20 executed, and verbosely printing any system calls that are executed.\n\
             To step backwards (i.e. back in time), use `{}`.",
            "back".bold(),
        ),
        |state, _label, _args| {
            if state.exited {
                return Err(CommandError::ProgramExited);
            }

            loop {
                let binary  = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
                let runtime = state.runtime.as_ref().ok_or(CommandError::MustLoadFile)?;

                let syscall = if let Ok(inst) = runtime.next_inst() {
                    util::print_inst(&state.iset, binary, inst, runtime.state().get_pc());

                    inst == 0xC
                } else {
                    false
                };

                let step = state.step(true)?;

                if step || syscall {
                    break;
                }
                
            }

            Ok(())
        }
    )
}
