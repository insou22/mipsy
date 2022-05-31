use crate::interactive::{error::CommandError, prompt};

use super::*;
use colored::*;

pub(crate) fn breakpoints_command() -> Command {
    command(
        "breakpoints",
        vec!["bs", "brs", "brks", "breaks"],
        vec![],
        vec![],
        "lists currently set breakpoints",
        &format!(
            "Lists currently set breakpoints.\n\
             When running or stepping through your program, a breakpoint will cause execution to\n\
         \x20 pause temporarily, allowing you to debug the current state.\n\
             May error if provided a label that doesn't exist.\n\
           \n{}{} you can also use the `{}` MIPS instruction in your program's code!",
             "tip".yellow().bold(),
             ":".bold(),
             "break".bold(),
        ),
        |state, _label, _args| {
            let binary = state.binary.as_mut().ok_or(CommandError::MustLoadFile)?;

            if binary.breakpoints.is_empty() {
                prompt::error_nl("no breakpoints set");
                return Ok(());
            }

            let mut breakpoints = binary.breakpoints.iter()
                    .map(|&addr| {
                        (
                            addr,
                            binary.labels.iter()
                                .find(|(_, &val)| val == addr)
                                .map(|(name, _)| (
                                    format!("{}", name.yellow().bold()),
                                    name.len()
                                ))
                        )
                    })
                    .collect::<Vec<(u32, Option<(String, usize)>)>>();

            breakpoints.sort_by_key(|(addr, _)| *addr);

            let max_len = breakpoints.iter()
                    .map(|(_, lbl)| {
                        lbl.as_ref()
                            .map(|(_, len)| *len)
                            .unwrap_or(0)
                    })
                    .max()
                    .unwrap_or(0);

            println!("\n{}", "[breakpoints]".green().bold());
            for (addr, text) in breakpoints {
                match text {
                    Some((name, len)) => {
                        println!("{}{} ({}{:08x})", name, " ".repeat(max_len - len), "0x".yellow(), addr);
                    }
                    None => {
                        println!("{}  {}{:08x}", " ".repeat(max_len), "0x".yellow(), addr);
                    }
                }
            }
            println!();

            Ok(())
        }
    )
}
