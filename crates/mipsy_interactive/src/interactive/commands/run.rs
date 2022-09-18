use crate::interactive::error::CommandError;

use super::*;
use colored::*;

pub(crate) fn run_command() -> Command {
    command(
        "run",
        vec!["r"],
        vec![],
        vec![],
        "run the currently loaded program until it finishes",
        |state, label, _args| {
            if label == "__help__" {
                return Ok(
                    format!(
                        "Runs the currently loaded program. It will run from wherever execution\n\
                     \x20 is currently (i.e. if you have used `{0}`, it will start from where you\n\
                     \x20 have currently stepped to).\n\
                         This will run in \"execution\" mode, {1} printing out instruction information,\n\
                     \x20 or other debug information that you would see while using `{0}`.",
                        "step".bold(),
                        "not".red().bold(),
                    ),
                )
            }

            if state.binary.is_none() {
                return Err(CommandError::MustLoadFile);
            }

            state.run()
        }
    )
}
