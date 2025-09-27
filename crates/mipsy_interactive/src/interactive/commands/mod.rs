#[allow(clippy::module_inception)]
pub(crate) mod util;

use std::fmt::Display;

pub(super) mod breakpoint;
pub(super) mod commands;
pub(super) mod context;
pub(super) mod disassemble;
pub(super) mod dot;
pub(super) mod examine;
pub(super) mod exit;
pub(super) mod help;
pub(super) mod label;
pub(super) mod labels;
pub(super) mod load;
pub(super) mod print;
pub(super) mod reset;
pub(super) mod run;
pub(super) mod step;
pub(super) mod watchpoint;

use super::{error::CommandResult, State};

// TODO: remove once if-let chaining is in
#[derive(Clone)]
pub(crate) enum ArgumentKind {
    Number(i64),
    File(String),
    Label(String),
    Item(String),
    Command(String),
    SubCommand(String),
    Any(String),
}

impl Display for ArgumentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number(_) => write!(f, "Number"),
            Self::File(_) => write!(f, "File"),
            Self::Label(_) => write!(f, "Label"),
            Self::Item(_) => write!(f, "Item"),
            Self::Command(_) => write!(f, "Command"),
            Self::SubCommand(_) => write!(f, "Sub command"),
            Self::Any(_) => write!(f, "Any"),
        }
    }
}

impl From<ArgumentKind> for String {
    fn from(value: ArgumentKind) -> Self {
        format!("{value}")
    }
}

// TODO: maybe this should be a trait actually. oh well
// TODO: remove once if-let chaining is in
#[derive(Clone)]
pub(crate) struct Argument {
    name: String,
    sanitiser: fn(arg: &str) -> CommandResult<ArgumentKind>,
}

impl Argument {
    fn new<S: Into<String>>(
        name: S,
        sanitiser: fn(arg: &str) -> CommandResult<ArgumentKind>,
    ) -> Self {
        Self {
            name: name.into(),
            sanitiser,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }
}

// TODO(joshh): remove once if-let chaining is in
#[derive(Clone)]
pub(crate) enum Arguments {
    Exactly {
        required: Vec<Argument>,
        // TODO: default values
        optional: Vec<Argument>,
    },
    VarArgs {
        required: Vec<Argument>,
        format: String,
    },
}

// TODO(joshh): remove once if-let chaining is in
#[derive(Clone)]
pub(crate) struct Command {
    pub(crate) names: Vec<String>,
    pub(crate) args: Arguments,
    description: String,
    _internal_exec: fn(&Command, &mut State, &str, &[ArgumentKind]) -> CommandResult<String>,
    subcommands: Vec<Command>,
}

impl Command {
    fn args_from_strings(&self, args: &[String]) -> CommandResult<Vec<ArgumentKind>> {
        let mut res = Vec::with_capacity(args.len());
        for arg in args.iter().flat_map(|strarg| match &self.args {
            Arguments::Exactly { required, optional } => required
                .iter()
                .map(|a| (a.sanitiser)(strarg))
                .chain(optional.iter().map(|a| (a.sanitiser)(strarg)))
                .collect::<Vec<CommandResult<ArgumentKind>>>(),
            Arguments::VarArgs { required, .. } => {
                required.iter().map(|a| (a.sanitiser)(strarg)).collect()
            }
        }) {
            res.push(arg?)
        }

        Ok(res)
    }

    pub(crate) fn exec(
        &self,
        state: &mut State,
        label: &str,
        args: &[String],
    ) -> CommandResult<String> {
        (self._internal_exec)(self, state, label, self.args_from_strings(args)?.as_slice())
    }

    pub(crate) fn new() -> Self {
        Self {
            names: Default::default(),
            args: Arguments::Exactly {
                required: Default::default(),
                optional: Default::default(),
            },
            description: Default::default(),
            _internal_exec: |_, _, _, _| Ok(Default::default()),
            subcommands: Default::default(),
        }
    }

    pub(crate) fn with_name<S: Into<String>>(mut self, name: S) -> Self {
        self.names.push(name.into());
        self
    }

    pub(crate) fn with_desc<S: Into<String>>(mut self, name: S) -> Self {
        self.description = name.into();
        self
    }

    pub(crate) fn with_subcommand(mut self, subcommands: Command) -> Self {
        self.subcommands.push(subcommands);
        self
    }

    pub(crate) fn with_exec(
        mut self,
        exec: fn(&Command, &mut State, &str, &[ArgumentKind]) -> CommandResult<String>,
    ) -> Self {
        self._internal_exec = exec;
        self
    }

    pub(crate) fn with_exact_args(mut self) -> Self {
        self.args = Arguments::Exactly {
            required: vec![],
            optional: vec![],
        };
        self
    }

    pub(crate) fn with_var_args(mut self) -> Self {
        self.args = Arguments::VarArgs {
            required: vec![],
            format: Default::default(),
        };
        self
    }

    pub(crate) fn with_required_arg(mut self, arg: Argument) -> Self {
        match self.args {
            Arguments::Exactly {
                ref mut required, ..
            } => required,
            Arguments::VarArgs {
                ref mut required, ..
            } => required,
        }
        .push(arg);

        self
    }

    pub(crate) fn with_optional_arg(mut self, arg: Argument) -> Self {
        match self.args {
            Arguments::Exactly {
                ref mut optional, ..
            } => optional.push(arg),
            Arguments::VarArgs { .. } => unreachable!(),
        }

        self
    }

    pub(crate) fn with_varargs_format(mut self, vfmt: impl Into<String>) -> Self {
        match self.args {
            Arguments::VarArgs { ref mut format, .. } => *format = vfmt.into(),
            Arguments::Exactly { .. } => unreachable!(),
        }
        self
    }
}
