use crate::interactive::prompt;

use super::*;
use colored::*;

pub(crate) fn command() -> Command {
    Command::new()
        .with_name("reset")
        .with_name("re")
        .with_desc("reset the currently loaded program to its initial state")
        .with_exec(|_, state, label, _args| {
            if label == "__help__" {
                return Ok(format!(
                    "Resets the currently loaded program to its inital state. This is\n\
                     \x20 effectively the same as using `{} {}` using the same file again.\n\
                         It is often used after `{}` or `{}` have reached the end of the program.",
                    "load".bold(),
                    "<file>".magenta(),
                    "run".bold(),
                    "step".bold(),
                ));
            }

            state.reset()?;
            prompt::success_nl("program reset");

            Ok("".into())
        })
}
