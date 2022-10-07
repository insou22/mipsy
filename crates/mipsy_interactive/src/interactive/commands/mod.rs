mod back;
mod breakpoint;
mod context;
mod commands;
mod disassemble;
mod dot;
mod examine;
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
mod watchpoint;

pub(crate) use back::back_command;
pub(crate) use breakpoint::breakpoint_command;
pub(crate) use context::context_command;
pub(crate) use disassemble::disassemble_command;
pub(crate) use dot::dot_command;
pub(crate) use examine::examine_command;
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
pub(crate) use watchpoint::watchpoint_command;

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
    description: String,
    _internal_exec: fn(&Command, &mut State, &str, &[String]) -> CommandResult<String>,
    subcommands: Vec<Command>,
}

impl Command {
    pub(crate) fn exec(&self, state: &mut State, label: &str, args: &[String]) -> CommandResult<String> {
        (self._internal_exec)(self, state, label, args)
    }
}

pub(crate) fn command<S: Into<String>>(name: S, aliases: Vec<S>, required_args: Vec<S>, optional_args: Vec<S>, subcommands: Vec<Command>, desc: S, exec: fn(&Command, &mut State, &str, &[String]) -> CommandResult<String>) -> Command {
    Command {
        name: name.into(),
        description: desc.into(),
        aliases: aliases.into_iter().map(S::into).collect(),
        args: Arguments::Exactly {
            required: required_args.into_iter().map(S::into).collect(),
            optional: optional_args.into_iter().map(S::into).collect(),
        },
        _internal_exec: exec,
        subcommands,
    }
}

pub(crate) fn command_varargs<S: Into<String>>(name: S, aliases: Vec<S>, required_args: Vec<S>, varargs_format: impl Into<String>, subcommands: Vec<Command>, desc: S, exec: fn(&Command, &mut State, &str, &[String]) -> CommandResult<String>) -> Command {
    Command {
        name: name.into(),
        description: desc.into(),
        aliases: aliases.into_iter().map(S::into).collect(),
        args: Arguments::VarArgs { required: required_args.into_iter().map(S::into).collect(), format: varargs_format.into() },
        _internal_exec: exec,
        subcommands,
    }
}
