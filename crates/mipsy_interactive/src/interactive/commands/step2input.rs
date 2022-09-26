use std::sync::atomic::Ordering;

use crate::interactive::error::CommandError;

use super::*;
use super::Command;
use colored::*;
use mipsy_lib::Register;

pub(crate) fn step2input_command() -> Command {
    command(
        "step2input",
        vec!["s2i"],
        vec![],
        vec![],
        vec![],
        "step forwards until next input",
        |_, state, label, _args| {
            if label == "__help__" {
                return Ok(
                    format!(
                        "Steps forwards until your program asks for its next input, or finishes.\n\
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

                let input = if let Ok(inst) = runtime.next_inst() {
                    util::print_inst(&state.iset, binary, inst, runtime.timeline().state().pc(), state.program.as_deref());
                    
                    if inst == 0xC {
                        let syscall = runtime.timeline().state().read_register(Register::V0.to_u32()).unwrap_or(-1);
                        matches!(syscall, 5 | 6 | 7 | 8 | 12)  
                    } else {
                        false
                    }
                } else {
                    false
                };

                let step = state.step(true)?;
                
                if step || input {
                    break;
                }
                
            }

            Ok("".into())
        }
    )
}
