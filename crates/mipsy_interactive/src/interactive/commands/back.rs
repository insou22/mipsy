use crate::interactive::{error::CommandError, prompt};

use super::*;
use colored::*;
use util::expect_u32;

pub(crate) fn back_command() -> Command {
    command(
        "back",
        vec!["b"],
        vec![],
        vec!["times"],
        vec![],
        &format!("step backwards one (or {}) instruction", "[times]".magenta()),
        |state, label, args| {
            if label == "__help__" {
                return Ok(
                    format!(
                        "Steps backwards one instruction, or {0} instructions if specified.\n\
                         It will then print out which instruction will be executed next --\n\
                     \x20 i.e. using `{1}` will immediately execute said printed instruction.\n\
                         To step fowards (i.e. normal stepping), use `{1}`.",
                        "[times]".magenta(),
                        "step".bold(),
                    ),
                )
            }

            let times = match args.first() {
                Some(arg) => expect_u32(
                    label,
                    &"[times]".bright_magenta(),
                    arg, 
                    Some(|neg: i32|
                        format!("try `{}{}`", "step ".bold(), (-neg).to_string().bold())
                    )
                ),
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

            let binary  = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
            let runtime = &state.runtime;

            let pluralise = if backs != 1 { "s" } else { "" };

            let mut text = format!("stepped back {} instruction{}", backs.to_string().magenta(), pluralise);
            
            if ran_out_of_history {
                text.push_str(" (before running out of history)");
            } else if backs < times {
                text.push_str(" (reached start of program)");
            }
            text.push_str(", next instruction will be:");

            prompt::success(text);
            if let Ok(inst) = runtime.next_inst() {
                util::print_inst(&state.iset, binary, inst, runtime.timeline().state().pc(), state.program.as_deref());
            }
            println!();

            Ok("".into())
        }
    )
}
