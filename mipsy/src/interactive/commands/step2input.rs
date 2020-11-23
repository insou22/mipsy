use crate::interactive::error::CommandError;

use super::*;
use super::{Command};
use colored::*;
use mipsy_lib::Register;

pub(crate) fn step2input_command() -> Command {
    command(
        "step2input",
        vec!["s2i"],
        vec![],
        vec![],
        "step forwards until next input",
        &format!(
            "Steps forwards until your program asks for its next input, or finishes.\n\
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

                let input = if let Ok(inst) = runtime.next_inst() {
                    util::print_inst(&state.iset, binary, inst, runtime.state().get_pc(), state.program.as_deref());
                    
                    if inst == 0xC {
                        let syscall = runtime.state().get_reg(Register::V0.to_u32()).unwrap_or(-1);
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

            Ok(())
        }
    )
}
