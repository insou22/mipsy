use crate::interactive::{error::CommandError, prompt};

use super::*;
use colored::*;

pub(crate) fn command() -> Command {
    Command::new()
        .with_name("label")
        .with_name("la")
        .with_name("lbl")
        .with_exact_args()
        // TODO: somehow get some context to sanitise & hint this
        .with_required_arg(Argument::new(
            "label",
            |a| Ok(ArgumentKind::Label(a.to_owned())),
            |_, _| vec![],
        ))
        .with_desc("print the address of a label")
        .with_exec(|_, state, label, args| {
            if label == "__help__" {
                return Ok(format!(
                    "Prints the address of the specified {0}.\n\
                         May error if the specified {0} doesn't exist.",
                    "<label>".magenta()
                ));
            }

            let ArgumentKind::Label(label) = &args[0] else {
                unreachable!()
            };
            let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;

            match binary.get_label(label) {
                Ok(addr) => {
                    prompt::success_nl(format!("{} => 0x{:08x}", label.yellow().bold(), addr))
                }
                Err(_) => prompt::error_nl(format!("could not find label \"{}\"", label)),
            }

            Ok("".into())
        })
}
