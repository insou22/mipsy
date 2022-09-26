use crate::interactive::{error::CommandError, prompt};

use super::*;
use colored::*;

pub(crate) fn label_command() -> Command {
    command(
        "label",
        vec!["la", "lbl"],
        vec!["label"],
        vec![],
        vec![],
        "print the address of a label",
        |state, label, args| {
            if label == "__help__" {
                return Ok(
                    format!(
                        "Prints the address of the specified {0}.\n\
                         May error if the specified {0} doesn't exist.",
                         "<label>".magenta()
                    ),
                )
            }

            let label = &args[0];
            let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;

            match binary.get_label(label) {
                Ok(addr) => prompt::success_nl(format!("{} => 0x{:08x}", label.yellow().bold(), addr)),
                Err(_)   => prompt::error_nl(format!("could not find label \"{}\"", label)),
            }

            Ok("".into())
        }
    )
}
