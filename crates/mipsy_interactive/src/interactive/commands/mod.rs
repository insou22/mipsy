mod back;
mod breakpoint;
mod context;
mod commands;
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
mod watchpoint;

pub(crate) use back::back_command;
pub(crate) use breakpoint::breakpoint_command;
pub(crate) use context::context_command;
pub(crate) use commands::commands_command;
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
pub(crate) use watchpoint::watchpoint_command;

use colored::Colorize;
use mipsy_lib::Register;
use std::fmt::Display;

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
    pub(crate) exec: fn(&mut State, &str, &[String]) -> CommandResult<String>,
}

pub(crate) fn command<S: Into<String>>(name: S, aliases: Vec<S>, required_args: Vec<S>, optional_args: Vec<S>, desc: S, exec: fn(&mut State, &str, &[String]) -> CommandResult<String>) -> Command {
    Command {
        name: name.into(),
        description: desc.into(),
        aliases: aliases.into_iter().map(S::into).collect(),
        args: Arguments::Exactly {
            required: required_args.into_iter().map(S::into).collect(),
            optional: optional_args.into_iter().map(S::into).collect(),
        },
        exec,
    }
}

pub(crate) fn command_varargs<S: Into<String>>(name: S, aliases: Vec<S>, required_args: Vec<S>, varargs_format: impl Into<String>, desc: S, exec: fn(&mut State, &str, &[String]) -> CommandResult<String>) -> Command {
    Command {
        name: name.into(),
        description: desc.into(),
        aliases: aliases.into_iter().map(S::into).collect(),
        args: Arguments::VarArgs { required: required_args.into_iter().map(S::into).collect(), format: varargs_format.into() },
        exec,
    }
}

#[derive(Copy, Clone)]
pub(crate) enum TargetAction {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

impl PartialEq for TargetAction {
    // eq is used to check if a watchpoint should trigger based on the action,
    // so a watchpoint checking for both reads and writes should always trigger
    fn eq(&self, other: &Self) -> bool {
        match *self {
            TargetAction::ReadWrite => true,
            _ => match other {
                    TargetAction::ReadWrite => true,
                    _ => core::mem::discriminant(self) == core::mem::discriminant(other),
            }
        }
    }
}

impl Display for TargetAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            match *self {
                TargetAction::ReadOnly  => "read",
                TargetAction::WriteOnly => "write",
                TargetAction::ReadWrite => "read/write",
            }
        )
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum WatchpointTarget {
    Register(Register),
    MemAddr(u32),
}

impl Display for WatchpointTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}",
            match self {
                WatchpointTarget::Register(r) => r.to_string(),
                WatchpointTarget::MemAddr(m)  => format!("{}{:08x}", "0x".yellow(), m),
            }
        )
    }
}

#[derive(PartialEq)]
pub(crate) struct TargetWatch {
    pub(crate) target: WatchpointTarget,
    pub(crate) action: TargetAction,
}

pub(crate) struct Watchpoint {
    pub(crate) id: u32,
    pub(crate) action: TargetAction,
    pub(crate) ignore_count: u32,
    pub(crate) enabled: bool,
}

impl Watchpoint {
    pub(crate) fn new(id: u32, action: TargetAction) -> Self {
        Self {
            id,
            action,
            ignore_count: 0,
            enabled: true,
        }
    }
}

