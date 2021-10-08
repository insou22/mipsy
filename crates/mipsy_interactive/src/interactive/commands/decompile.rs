use crate::interactive::error::CommandError;

use super::*;
use colored::*;

use mipsy_lib::decompile::{Decompiled, Uninit, decompile_into_parts};

pub(crate) fn decompile_command() -> Command {
    command(
        "decompile",
        vec!["d", "dec"],
        vec![],
        vec![],
        "decompiles the currently loaded file",
        &format!(
            "Decompiles the currently loaded file, similar to how `{}` displays instructions.",
            "step".bold(),
        ),
        |state, _label, _args| {
            let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;

            let mut decompiled = decompile_into_parts(binary, &state.iset)
                    .into_iter()
                    .collect::<Vec<(u32, Result<Decompiled, Uninit>)>>();
            
            decompiled.sort_by_key(|&(addr, _)| addr);

            if let Some((_, inst)) = decompiled.get(0) {
                let labels = match inst {
                    Ok(ok)   => &ok.labels,
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

            Ok(())
        }
    )
}
