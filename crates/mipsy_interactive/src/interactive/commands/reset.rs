use crate::interactive::prompt;

use super::*;
use colored::*;

pub(crate) fn reset_command() -> Command {
    command(
        "reset",
        vec!["re"],
        vec![],
        vec![],
        vec![],
        "reset the currently loaded program to its initial state",
        |_, state, label, _args| {
            if label == "__help__" {
                return Ok(
                    format!(
                        "Resets the currently loaded program to its inital state. This is\n\
                     \x20 effectively the same as using `{} {}` using the same file again.\n\
                         It is often used after `{}` or `{}` have reached the end of the program.",
                        "load".bold(),
                        "<file>".magenta(),
                        "run".bold(),
                        "step".bold(),
                    ),
                )
            }

            state.reset()?;
            prompt::success_nl("program reset");

            Ok("".into())
        }
    )
}
