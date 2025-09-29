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

use crate::interactive::helper::{HintArgs, MyHelper};

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

// TODO: another callback for __help__ label?
// TODO: remove once if-let chaining is in
#[derive(Clone)]
pub(crate) struct Argument {
    name: String,
    sanitiser: fn(arg: &str) -> CommandResult<ArgumentKind>,
    hints: fn(harg: &HintArgs, helper: &MyHelper) -> Vec<String>,
}

impl Argument {
    fn new<S: Into<String>>(
        name: S,
        sanitiser: fn(arg: &str) -> CommandResult<ArgumentKind>,
        hints: fn(harg: &HintArgs, helper: &MyHelper) -> Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            sanitiser,
            hints,
        }
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn hints(&self, mut harg: HintArgs, helper: &MyHelper) -> Vec<String> {
        helper
            .closest_hints(
                &(self.hints)(&harg, helper)
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                &harg.subcmd(),
            )
            .iter()
            .map(|s| harg.recontextualize(s))
            .collect::<Vec<_>>()
    }

    pub(crate) fn subcommands() -> Self {
        Argument::new(
            "subcommand",
            |a| {
                // TODO: match with subcmd hints
                Ok(ArgumentKind::SubCommand(a.to_owned()))
            },
            |_, _| vec![],
        )
    }
}

// TODO(joshh): remove once if-let chaining is in
#[derive(Clone)]
pub(crate) enum Arguments {
    Exactly {
        required: Vec<Argument>,
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

    pub(crate) fn args(&self) -> Vec<&Argument> {
        match &self.args {
            Arguments::Exactly { required, optional } => {
                required.iter().chain(optional.iter()).collect::<Vec<_>>()
            }
            // TODO: varargs
            Arguments::VarArgs { required, .. } => required.iter().collect(),
        }
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

    pub(crate) fn name(&self) -> &str {
        &self.names.get(0).expect("command has no name")
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
