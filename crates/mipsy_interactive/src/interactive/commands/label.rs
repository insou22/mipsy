use crate::interactive::{error::CommandError, prompt};

use super::*;
use colored::*;

pub(crate) fn command() -> Command {
    Command::new()
        .with_name("label")
        .with_name("la")
        .with_name("lbl")
        .with_exact_args()
        .with_required_arg(Argument::new(
            "label",
            |_, a, h| {
                h.state
                    .binary
                    .as_ref()
                    .ok_or(CommandError::MustLoadFile)?
                    .get_label(a)
                    .map_err(|_| CommandError::BadArgument {
                        arg: "<label>".magenta().to_string(),
                        instead: a.to_owned(),
                    })
                    // have to keep it as a string here because
                    // it gets printed later
                    .and_then(|_| Ok(ArgumentKind::String(a.to_owned())))
            },
            |_, _, h| {
                h.state
                    .binary
                    .as_ref()
                    .and_then(|b| {
                        Some(
                            b.labels
                                .keys()
                                .filter(|k| !(k.starts_with("kernel__") || k.starts_with("_start")))
                                .cloned()
                                .collect(),
                        )
                    })
                    .unwrap_or_default()
            },
        ))
        .with_desc("print the address of a label")
        .with_help(format!(
            "Prints the address of the specified {0}.\n\
                May error if the specified {0} doesn't exist.",
            "<label>".magenta()
        ))
        .with_exec(|_, helper, args| {
            let label = String::from(args[0].to_owned());
            let binary = helper
                .state
                .binary
                .as_ref()
                .ok_or(CommandError::MustLoadFile)?;

            match binary.get_label(&label) {
                Ok(addr) => {
                    prompt::success_nl(format!("{} => 0x{:08x}", label.yellow().bold(), addr))
                }
                Err(_) => prompt::error_nl(format!("could not find label \"{}\"", label)),
            }

            Ok("".into())
        })
}
