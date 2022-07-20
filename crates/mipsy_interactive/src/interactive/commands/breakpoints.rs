use crate::interactive::{error::CommandError, prompt};

use super::*;
use colored::*;

// TODO(joshh): remove and possibly turn into alias
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
        |state, _label, _args| {todo!()}
    )
}
