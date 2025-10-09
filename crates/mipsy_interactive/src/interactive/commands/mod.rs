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
        match value {
            ArgumentKind::String(s) => s,
            ArgumentKind::Number(_) => unreachable!("tried to interpret as a string but argument was sanitised into a number")
        }
    }
}

impl From<ArgumentKind> for i64 {
    fn from(value: ArgumentKind) -> Self {
        match value {
            ArgumentKind::Number(n) => n,
            ArgumentKind::String(_) => unreachable!("tried to interpret as a number but argument was sanitised into a string")
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

    pub(crate) fn from_name(name: impl Into<String>) -> Self {
        Argument::new(
            name,
            |_, a, _| Ok(ArgumentKind::String(a.to_owned())),
            |_, _, _| vec![],
        )
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

    // TODO: remove command param from callback and have a seperate argument
    //       allowing us to forward arguments to subcommand params
    pub(crate) fn subcommands() -> Self {
        Argument::new(
            "subcommand",
            |c, a, _| match c
                .subcommands
                .iter()
                .flat_map(|c| &c.names)
                .find(|&s| s == a)
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
        variadic: Argument,
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
    fn sanitise_args(
        &self,
        args: &[String],
        helper: &MyHelper,
    ) -> CommandResult<Vec<ArgumentKind>> {
        args.iter()
            .zip(match &self.args {
                Arguments::Exactly { required, optional } => required
                    .iter()
                    .map(|a| a.sanitiser)
                    .chain(optional.iter().map(|a| a.sanitiser))
                    // TODO: this isnt great, we assume the last argument is a subcommand
                    // and forward its args. not an ideal solution.
                    //       a better way would be to have an explicit subcommand
                    //       argument type but that would make the usual normal arguments
                    //       clunky
                    .chain(std::iter::repeat_n(
                        (|_, a, _| Ok(ArgumentKind::String(a.to_owned())))
                        as fn(cmd: &Command, arg: &str, helper: &MyHelper) -> CommandResult<ArgumentKind>
                    , args.len() - required.len()))
                    .collect::<Vec<_>>(),
                Arguments::VarArgs { required, variadic } => required
                    .iter()
                    .map(|a| a.sanitiser)
                    .chain(std::iter::repeat_n(
                        variadic.sanitiser,
                        (args.len() - required.len()) + 1,
                    ))
                    .collect(),
            })
            // TODO: give varargs the whole arg instead of `strarg`
            .map(|(strarg, san)| san(self, strarg, helper))
            .collect()
    }

    pub(crate) fn args(&self, vararg_count: usize) -> Vec<&Argument> {
        match &self.args {
            Arguments::Exactly { required, optional } => {
                required.iter().chain(optional.iter()).collect::<Vec<_>>()
            }
            Arguments::VarArgs { required, variadic } => required
                .iter()
                .chain(std::iter::repeat_n(variadic, vararg_count))
                .collect(),
        }
    }

    pub(super) fn required_args(&self) -> &[Argument] {
        match &self.args {
            Arguments::Exactly { required, .. } => required,
            Arguments::VarArgs { required, .. } => required,
        }
    }

    pub(crate) fn exec(&self, helper: &mut MyHelper, args: &[String]) -> CommandResult<String> {
        if args.len() < self.required_args().len() {
            Err(CommandError::WithTip {
                error: Box::new(CommandError::MissingArguments {
                    args: self.required_args()
                        .iter()
                        .map(Argument::name)
                        .map(str::to_owned)
                        .collect(),
                    instead: args.to_vec(),
                }),
                tip: format!("try `{} {}`", "help".bold(), self.name().bold()),
            })
        } else {
            // TODO this is not great, it assumes that the last argument is a subcommand
            // and will forward all the arguments to them

            (self._internal_exec)(
                self,
                helper,
                &self.sanitise_args(args, &helper)?
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

    /// NOTE: resets the argument kind to [`Arguments::VarArgs`]
    ///       this resets any
    ///         - [`Arguments::VarArgs::required`]
    ///         - [`Arguments::Exactly::required`]
    ///         - [`Arguments::Exactly::optional`]
    ///       Argumets, so call this before adding any other arguments
    ///       \
    ///       The [`Argument::name`] of `arg` is set to the vararg format
    ///       and the [`Argument::sanitiser`] and [`Argument::hints`] for variadic arguments
    pub(crate) fn with_var_args(mut self, arg: Argument) -> Self {
        self.args = Arguments::VarArgs {
            required: vec![],
            variadic: arg,
        };
        self
    }

    pub(crate) fn with_exact_args(mut self) -> Self {
        self.args = Arguments::Exactly {
            required: vec![],
            optional: vec![],
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
            Arguments::VarArgs { .. } => unreachable!("tried to supply an optional argument while being in the VarArgs argument state\ntry use `self.with_exact_args()` to set the argument state"),
        }

        self
    }
}
