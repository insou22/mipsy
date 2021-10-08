use crate::interactive::error::CommandError;

use super::*;
use super::{Command, util::expect_u32};
use colored::*;

pub(crate) fn step_command() -> Command {
    command(
        "step",
        vec!["s"],
        vec![],
        vec!["times"],
        &format!("step forwards one (or {}) instruction", "[times]".magenta()),
        &format!(
            "Steps forwards one instruction, or {} instructions if specified.\n\
             This will run in \"verbose\" mode, printing out the instruction that was\n\
         \x20 executed, and verbosely printing any system calls that are executed.\n\
             To step backwards (i.e. back in time), use `{}`.",
            "[times]".magenta(),
            "back".bold(),
        ),
        |state, label, args| {
            let times = match args.first() {
                Some(arg) => expect_u32(
                    label,
                    &"[times]".bright_magenta().to_string(),
                    arg, 
                    Some(|neg: i32|
                        format!("try `{}{}`", "back ".bold(), (-neg).to_string().bold())
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
                    util::print_inst(&state.iset, binary, inst, runtime.timeline().state().pc(), state.program.as_deref());
                }

                let step = state.step(true)?;

                if step {
                    break;
                }
            }

            Ok(())
        }
    )
}
