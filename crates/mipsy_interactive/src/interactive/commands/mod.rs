#[allow(clippy::module_inception)]
pub(crate) mod util;

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

use colored::Colorize;

use crate::interactive::{
    error::CommandError,
    helper::{HintArgs, MyHelper},
};

use super::{error::CommandResult, State};

// TODO: remove once if-let chaining is in
#[derive(Clone, Debug)]
pub(crate) enum ArgumentKind {
    Number(i64),
    String(String),
}

impl From<ArgumentKind> for String {
    fn from(value: ArgumentKind) -> Self {
        if let ArgumentKind::String(s) = value {
            s
        } else {
            unreachable!()
        }
    }
}

impl From<ArgumentKind> for i64 {
    fn from(value: ArgumentKind) -> Self {
        if let ArgumentKind::Number(n) = value {
            n
        } else {
            unreachable!()
        }
    }
}

// TODO: remove cmd callback params. find another way because currently its only use is for getting subcommands
// TODO: remove once if-let chaining is in
#[derive(Clone, Debug)]
pub(crate) struct Argument {
    name: String,
    sanitiser: fn(cmd: &Command, arg: &str, helper: &MyHelper) -> CommandResult<ArgumentKind>,
    hints: fn(cmd: &Command, harg: &HintArgs, helper: &MyHelper) -> Vec<String>,
}

impl Argument {
    fn new<S: Into<String>>(
        name: S,
        sanitiser: fn(cmd: &Command, arg: &str, helper: &MyHelper) -> CommandResult<ArgumentKind>,
        hints: fn(cmd: &Command, harg: &HintArgs, helper: &MyHelper) -> Vec<String>,
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

    pub(crate) fn hints(&self, cmd: &Command, harg: &HintArgs, helper: &MyHelper) -> Vec<String> {
        helper
            .closest_hints(
                &(self.hints)(cmd, &harg, helper)
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>(),
                &harg.subcmd(),
            )
            .iter()
            .map(|s| harg.recontextualize(s))
            .collect()
    }

    pub(crate) fn subcommands() -> Self {
        Argument::new(
            "subcommand",
            |c, a, _| match c
                .subcommands
                .iter()
                .flat_map(|c| &c.names)
                .find(|&s| s == &a.to_owned())
            {
                Some(_) => Ok(ArgumentKind::String(a.to_owned())),
                None => Err(CommandError::BadArgument {
                    arg: "any subcommand".to_owned(),
                    instead: a.to_owned(),
                }),
            },
            |c, _, _| {
                c.subcommands
                    .iter()
                    .map(Command::name)
                    .map(str::to_owned)
                    .collect()
            },
        )
    }
}

// TODO(joshh): remove once if-let chaining is in
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub(crate) struct Command {
    pub(crate) names: Vec<String>,
    pub(crate) args: Arguments,
    description: String,
    help: String,
    _internal_exec: fn(&Command, &mut MyHelper, &[ArgumentKind]) -> CommandResult<String>,
    subcommands: Vec<Command>,
}

impl Command {
    fn args_from_strings(
        &self,
        args: &[String],
        helper: &MyHelper,
    ) -> CommandResult<Vec<ArgumentKind>> {
        let mut res = Vec::with_capacity(args.len());
        for arg in args.iter().flat_map(|strarg| match &self.args {
            Arguments::Exactly { required, optional } => required
                .iter()
                .map(|a| (a.sanitiser)(self, strarg, helper))
                .chain(optional.iter().map(|a| (a.sanitiser)(self, strarg, helper)))
                .collect::<Vec<CommandResult<ArgumentKind>>>(),
            Arguments::VarArgs { required, .. } => required
                .iter()
                .map(|a| (a.sanitiser)(self, strarg, helper))
                .collect(),
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

    pub(super) fn required_args(&self) -> &[Argument] {
        match &self.args {
            Arguments::Exactly { required, .. } => required,
            Arguments::VarArgs { required, .. } => required,
        }
    }

    pub(crate) fn exec(&self, helper: &mut MyHelper, args: &[String]) -> CommandResult<String> {
        let required = self.required_args();
        if args.len() < required.len() {
            Err(CommandError::WithTip {
                error: Box::new(CommandError::MissingArguments {
                    args: required
                        .iter()
                        .map(Argument::name)
                        .map(str::to_owned)
                        .collect(),
                    instead: args.to_vec(),
                }),
                tip: format!("try `{} {}`", "help".bold(), self.name().bold()),
            })
        } else {
            (self._internal_exec)(
                self,
                helper,
                self.args_from_strings(args, &helper)?.as_slice(),
            )
        }
    }

    pub(crate) fn new() -> Self {
        Self {
            names: Default::default(),
            args: Arguments::Exactly {
                required: Default::default(),
                optional: Default::default(),
            },
            description: Default::default(),
            help: Default::default(),
            _internal_exec: |_, _, _| Ok(Default::default()),
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

    pub(crate) fn with_desc<S: Into<String>>(mut self, desc: S) -> Self {
        self.description = desc.into();
        self
    }

    pub(crate) fn with_help<S: Into<String>>(mut self, help: S) -> Self {
        self.help = help.into();
        self
    }

    pub(crate) fn with_subcommand(mut self, subcommands: Command) -> Self {
        self.subcommands.push(subcommands);
        self
    }

    pub(crate) fn with_exec(
        mut self,
        exec: fn(&Command, helper: &mut MyHelper, &[ArgumentKind]) -> CommandResult<String>,
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
