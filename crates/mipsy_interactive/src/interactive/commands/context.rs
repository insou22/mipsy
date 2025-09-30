use super::*;
use crate::interactive::{commands::util::expect_u32, error::CommandError};
use colored::*;
use mipsy_lib::decompile;
use mipsy_lib::KTEXT_BOT;
use mipsy_lib::TEXT_BOT;

#[allow(unreachable_code)]
pub(crate) fn command() -> Command {
    Command::new()
        .with_name("context")
        .with_name("c")
        .with_name("ctx")
        .with_exact_args()
        .with_optional_arg(Argument::new(
            "n",
            |a, _| {
                Ok(ArgumentKind::Number(expect_u32(
                    "",
                    &"[n]".bright_magenta(),
                    a,
                    None as Option<&dyn Fn(i32) -> String>,
                )? as _))
            },
            |_, _| vec![],
        ))
        .with_desc(format!(
            "prints the current and surrounding 3 (or {}) instructions",
            "[n]".magenta(),
        ))
        .with_help(format!(
            "prints the current and surrounding 3 (or {}) instructions",
            "[n]".magenta(),
        ))
        .with_exec(|_, state, _, args| {
            let n = match args.first() {
                Some(ArgumentKind::Number(a)) => *a as _,
                None => 3,
                _ => unreachable!(),
            };

            if state.exited {
                return Err(CommandError::ProgramExited);
            }

            let program = state.program.as_ref().ok_or(CommandError::MustLoadFile)?;
            let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
            let runtime = &state.runtime;

            let base_addr = runtime.timeline().state().pc();
            for i in (-n)..=n {
                let addr = {
                    let addr = base_addr.wrapping_add((i * 4) as u32);
                    if addr < TEXT_BOT {
                        continue;
                    }

                    if addr < KTEXT_BOT && addr >= (TEXT_BOT + binary.text.len() as u32) {
                        continue;
                    }

                    addr
                };

                let inst = {
                    if let Ok(inst) = runtime.timeline().state().read_mem_word(addr) {
                        inst
                    } else {
                        continue;
                    }
                };

                let parts = decompile::decompile_inst_into_parts(binary, &state.iset, inst, addr);
                util::print_inst_parts(binary, &Ok(parts), Some(program), i == 0);
            }

            println!();
            Ok("".into())
        })
}
