use mipsy_lib::decompile;
use crate::KTEXT_BOT;
use crate::TEXT_BOT;
use crate::interactive::{error::CommandError, commands::util::expect_u32};
use super::*;
use colored::*;

#[allow(unreachable_code)]
pub(crate) fn context_command() -> Command {
    command(
        "context",
        vec!["c", "ctx"],
        vec![],
        vec!["n"],
        &format!(
            "prints the current and surrounding 3 (or {}) instructions",
            "[n]".magenta(),
        ),
        &format!(
            "prints the current and surrounding 3 (or {}) instructions",
            "[n]".magenta(),
        ),
        |state, label, args| {
            let f: Option<&dyn Fn(i32) -> String> = None;

            let n = match args.first() {
                Some(arg) => expect_u32(
                    label,
                    &"[n]".bright_magenta().to_string(),
                    arg,
                    f
                ),
                None => Ok(3),
            }? as i32;

            if state.exited {
                return Err(CommandError::ProgramExited);
            }

            let program = state.program.as_ref().ok_or(CommandError::MustLoadFile)?;
            let binary  = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
            let runtime = state.runtime.as_ref().ok_or(CommandError::MustLoadFile)?;

            let base_addr = runtime.state().get_pc();
            for i in (-n)..=n {
                let addr = {
                    let addr = base_addr.wrapping_add((i * 4) as u32);
                    if addr < TEXT_BOT {
                        continue;
                    }

                    if addr < KTEXT_BOT && addr >= (TEXT_BOT + (binary.text.len() * 4) as u32) {
                        continue;
                    }

                    addr
                };

                let inst = {
                    if let Ok(inst) = runtime.state().get_word(addr) {
                        inst
                    } else {
                        continue;
                    }
                };

                let parts = decompile::decompile_inst_into_parts(binary, &state.iset, inst, addr);
                util::print_inst_parts(binary, &parts, Some(program), i == 0);
            }

            Ok(())
        }
    )
}
