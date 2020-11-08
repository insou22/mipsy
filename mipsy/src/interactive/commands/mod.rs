mod back;
mod breakpoint;
mod breakpoints;
mod decompile;
mod exit;
mod help;
mod load;
mod label;
mod labels;
mod print;
mod reset;
mod run;
mod step;
mod step2input;
mod step2syscall;
mod util;

pub(crate) use back::back_command;
pub(crate) use breakpoint::breakpoint_command;
pub(crate) use breakpoints::breakpoints_command;
pub(crate) use decompile::decompile_command;
pub(crate) use exit::exit_command;
pub(crate) use help::help_command;
pub(crate) use load::load_command;
pub(crate) use label::label_command;
pub(crate) use labels::labels_command;
pub(crate) use print::print_command;
pub(crate) use reset::reset_command;
pub(crate) use run::run_command;
pub(crate) use step::step_command;
pub(crate) use step2input::step2input_command;
pub(crate) use step2syscall::step2syscall_command;

use super::{error::CommandResult, State};

pub(crate) struct Command {
    pub(crate) name: String,
    pub(crate) aliases: Vec<String>,
    pub(crate) required_args: Vec<String>,
    pub(crate) optional_args: Vec<String>,
    pub(crate) description: String,
    pub(crate) long_description: String,
    pub(crate) exec: fn(&mut State, &str, &[String]) -> CommandResult<()>,
}

pub(crate) fn command<S: Into<String>>(name: S, aliases: Vec<S>, required_args: Vec<S>, optional_args: Vec<S>, desc: S, long_desc: S, exec: fn(&mut State, &str, &[String]) -> CommandResult<()>) -> Command {
    Command {
        name: name.into(),
        description: desc.into(),
        long_description: long_desc.into(),
        aliases: aliases.into_iter().map(S::into).collect(),
        required_args: required_args.into_iter().map(S::into).collect(),
        optional_args: optional_args.into_iter().map(S::into).collect(),
        exec,
    }
}
