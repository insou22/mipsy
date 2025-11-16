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
    Correct,
}

impl TryFrom<ArgumentKind> for String {
    type Error = String;
    fn try_from(value: ArgumentKind) -> Result<String, Self::Error> {
        match value {
            ArgumentKind::String(s) => Ok(s),
            _ => Err(format!(
                "tried to interpret argument as a string but argument was not sanitised into a string"
            )),
        }
    }
}

impl TryFrom<ArgumentKind> for i64 {
    type Error = String;
    fn try_from(value: ArgumentKind) -> Result<i64, String> {
        match value {
            ArgumentKind::Number(n) => Ok(n),
            _ => Err(format!(
                "tried to interpret argument as a number but argument was not sanitised into a number"
            )),
        }
    }
}

// TODO: remove cmd callback params. find another way because currently its only use is for getting subcommands
type Sanitiser = fn(arg: &str, helper: &MyHelper) -> CommandResult<ArgumentKind>;
type Hints = fn(harg: &HintArgs, helper: &MyHelper) -> Vec<String>;
// TODO: remove once if-let chaining is in
#[derive(Clone, Debug)]
pub(crate) enum Argument {
    Normal {
        name: String,
        sanitiser: Sanitiser,
        hints: Hints,
    },
    Subcommand,
}

impl Argument {
    fn new(name: impl Into<String>, sanitiser: Sanitiser, hints: Hints) -> Self {
        Self::Normal {
            name: name.into(),
            sanitiser,
            hints,
        }
    }

    fn sanitiser<'a, 'b: 'a>(
        &'a self,
        cmd: &'b Command,
    ) -> Box<dyn FnMut(&'a str, &'b MyHelper) -> CommandResult<ArgumentKind> + 'a> {
        match self {
            Argument::Normal { sanitiser, .. } => Box::new(sanitiser),
            Argument::Subcommand => Box::new(|a: &str, _| {
                match cmd
                    .subcommands
                    .iter()
                    .flat_map(|c| &c.names)
                    .find(|&s| s == &a)
                {
                    Some(_) => Ok(ArgumentKind::Correct),
                    None => Err(CommandError::BadArgument {
                        arg: "any subcommand".to_owned(),
                        instead: a.to_owned(),
                    }),
                }
            }),
        }
    }

    pub(crate) fn from_name(name: impl Into<String>) -> Self {
        Argument::new(
            name,
            |a, _| Ok(ArgumentKind::String(a.to_owned())),
            |_, _| vec![],
        )
    }

    pub(crate) fn name(&self) -> &str {
        match self {
            Self::Normal { name, .. } => name,
            Self::Subcommand => "subcommand",
        }
    }

    pub(crate) fn hints(&self, cmd: &Command, harg: &HintArgs, helper: &MyHelper) -> Vec<String> {
        helper
            .closest_hints(
                match self {
                    Self::Normal { hints, .. } => hints(&harg, helper),
                    Self::Subcommand => cmd
                        .subcommands
                        .iter()
                        .map(Command::name)
                        .map(str::to_owned)
                        .collect(),
                }
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .as_slice(),
                &harg.subcmd(),
            )
            .iter()
            .map(|s| harg.recontextualize(s))
            .collect()
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
        let mut optional_sans: Vec<_> = match &self.args {
            Arguments::Exactly { optional, .. } => {
                optional.iter().map(|arg| arg.sanitiser(self)).collect()
            }
            Arguments::VarArgs { required, variadic } => {
                std::iter::repeat_n(variadic, (args.len() - required.len()) + 1)
                    .map(|v| v.sanitiser(self))
                    .collect()
            }
        };

        // required args must be zipped with their sanitisers
        // optional args, however, are able to pick which sanitiser they work with
        // e.g.
        //      for 'step 1' and 'step back' comamnds, the count num and subcommand
        //      arguments are both optional, so when supplying 'back' it must choose
        //      a sanitiser which works (the subcommand sanitiser) instead of zipping
        //      which would result in 'back' being sanitised as the count num argument

        // only to please the borrow checker, this could be expressed more logically by
        // making `optional_sans` a mutable iterator if it were possible:(
        let mut skip = 0;

        println!("hi");
        args.iter()
            .zip(self.required_args())
            .map(|(strarg, arg)| (arg.sanitiser(self))(strarg, helper))
            .chain(
                args
                    .iter()
                    .skip(self.required_args().len())
                    // TODO: give varargs the whole arg instead of just one `strarg`
                    .map(|strarg| {
                        println!("{skip}");
                        let (skipped, san) = optional_sans
                            .iter_mut()
                            .map(|san| san(strarg, helper))
                            .skip(skip)
                            .enumerate()
                            .skip_while(|(_, san)| san.is_err())
                            .next()
                            .map_or(
                                (1, optional_sans.iter_mut().nth(skip).expect(
                                    "somehow there are not enough input sanitisers left (impossible)",
                                )(strarg, helper)),
                                |(skipped, san)| (skipped + 1, san)
                            );
                        skip += skipped;
                        san
                    }),
            )
            .collect()
    }

    pub(crate) fn args(&self, vararg_count: usize) -> Vec<&Argument> {
        match &self.args {
            Arguments::Exactly { required, optional } => {
                required.iter().chain(optional.iter()).collect()
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
                    args: self
                        .required_args()
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

            (self._internal_exec)(self, helper, &self.sanitise_args(args, &helper)?)
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
