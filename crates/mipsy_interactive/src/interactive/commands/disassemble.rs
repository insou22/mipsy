use crate::interactive::error::CommandError;

use super::*;
use colored::*;

use mipsy_lib::decompile::{decompile_into_parts, Decompiled, Uninit};

pub(crate) fn command() -> Command {
    Command::new()
        .with_name("disassemble")
        .with_name("d")
        .with_name("dis")
        .with_name("disas")
        .with_name("disasm")
        .with_name("dec")
        .with_name("decompile")
        .with_desc("disassembles the currently loaded file")
        .with_help(format!(
            "Disassembles the currently loaded file, similar to how `{}` displays instructions.",
            "step".bold(),
        ))
        .with_exec(|_, state, _, _| {
            let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;

            let mut decompiled = decompile_into_parts(binary, &state.iset)
                .into_iter()
                .collect::<Vec<(u32, Result<Decompiled, Uninit>)>>();

            decompiled.sort_by_key(|&(addr, _)| addr);

            if let Some((_, inst)) = decompiled.get(0) {
                let labels = match inst {
                    Ok(ok) => &ok.labels,
                    Err(err) => &err.labels,
                };

                if labels.is_empty() {
                    println!();
                }
            }

            for (_, inst) in decompiled {
                util::print_inst_parts(binary, &inst, state.program.as_deref(), false);
            }

            println!();

            Ok("".into())
        })
}
