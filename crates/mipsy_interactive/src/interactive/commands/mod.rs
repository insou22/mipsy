mod back;
mod breakpoint;
mod breakpoints;
mod context;
mod disassemble;
mod dot;
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
pub(crate) mod util;

pub(crate) use back::back_command;
pub(crate) use breakpoint::breakpoint_command;
pub(crate) use breakpoints::breakpoints_command;
pub(crate) use context::context_command;
pub(crate) use disassemble::disassemble_command;
pub(crate) use dot::dot_command;
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

// TODO(joshh): remove once if-let chaining is in
#[derive(Clone)]
pub(crate) enum Arguments {
    Exactly { required: Vec<String>, optional: Vec<String> },
    VarArgs { required: Vec<String>, format: String, },
}

// TODO(joshh): remove once if-let chaining is in
#[derive(Clone)]
pub(crate) struct Command {
    pub(crate) name: String,
    pub(crate) aliases: Vec<String>,
    pub(crate) args: Arguments,
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
        args: Arguments::Exactly {
            required: required_args.into_iter().map(S::into).collect(),
            optional: optional_args.into_iter().map(S::into).collect(),
        },
        exec,
    }
}

pub(crate) fn command_varargs<S: Into<String>>(name: S, aliases: Vec<S>, required_args: Vec<S>, varargs_format: impl Into<String>, desc: S, long_desc: S, exec: fn(&mut State, &str, &[String]) -> CommandResult<()>) -> Command {
    Command {
        name: name.into(),
        description: desc.into(),
        long_description: long_desc.into(),
        aliases: aliases.into_iter().map(S::into).collect(),
        args: Arguments::VarArgs { required: required_args.into_iter().map(S::into).collect(), format: varargs_format.into() },
        exec,
    }
}
