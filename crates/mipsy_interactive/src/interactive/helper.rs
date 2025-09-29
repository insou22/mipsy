use rustyline::{
    completion::{Candidate, Completer, FilenameCompleter, Pair},
    error::ReadlineError,
    highlight::Highlighter,
    hint::{Hinter, HistoryHinter},
    validate::{ValidationContext, ValidationResult, Validator},
    Context,
};
use rustyline_derive::Helper;
use std::{
    borrow::Cow::{self, Borrowed, Owned},
    ptr::NonNull,
};

use crate::interactive::commands::{Argument, ArgumentKind, Arguments, Command};

#[derive(Clone)]
pub(crate) struct HintArgs {
    pub(crate) line: String,
    pub(crate) pos: usize,
}

impl HintArgs {
    fn hints(&self, strs: &[&String]) -> (usize, Vec<Pair>) {
        (
            self.pos,
            strs.iter()
                .cloned()
                .map(|h| self.pair(h.to_owned()))
                .collect(),
        )
    }

    fn pair(&self, s: String) -> Pair {
        Pair {
            display: s.clone(),
            replacement: s[self.pos.min(s.len())..].to_owned(),
        }
    }

    fn subcmd_index(&self) -> usize {
        self.line
            .trim_start()
            .chars()
            .last()
            .unwrap_or_default()
            .is_whitespace() as usize
            + self.line[..self.pos].split_whitespace().skip(1).count()
    }

    pub(crate) fn subcmd(&self) -> Self {
        let line = self.line.split_whitespace().last().expect("impossible");
        Self {
            line: line.to_owned(),
            pos: line.len(),
        }
    }

    pub(crate) fn recontextualize(&self, sub: &str) -> String {
        self.line[..self.line.len() - self.subcmd().line.len()].to_owned() + &sub
    }
}

#[derive(Helper)]
pub(crate) struct MyHelper<'a> {
    completer: FilenameCompleter,
    hinter: HistoryHinter,
    pub(crate) commands: &'a [Command],
}

impl<'a> MyHelper<'a> {
    pub(super) fn new(commands: &'a [Command]) -> Self {
        Self {
            completer: FilenameCompleter::new(),
            hinter: HistoryHinter {},
            commands: commands,
        }
    }

    fn command(&self, line: &str) -> Option<&Command> {
        line.split_whitespace()
            .nth(0)
            .and_then(|c| self.commands.iter().find(|d| d.name() == c))
    }

    pub(crate) fn closest_hints<'b>(&self, with: &[&'b str], hargs: &HintArgs) -> Vec<&'b str> {
        with.iter()
            .filter(|m| m.len() != hargs.pos)
            .filter(|m| m.starts_with(&hargs.line))
            .copied()
            .collect()
    }

    fn closest_command(&self, hargs: &HintArgs) -> Option<String> {
        if hargs.line.is_empty() || hargs.pos < hargs.line.len() {
            None
        } else if let Some(found) = self
            .commands
            .iter()
            .find(|s| s.name().starts_with(&hargs.line))
        {
            let found = found.name();
            if found.len() == hargs.pos {
                None
            } else {
                Some(found[hargs.pos..].to_owned())
            }
        } else {
            None
        }
    }

    fn history_hints(&self, hargs: &HintArgs, ctx: &'a Context<'_>) -> Vec<String> {
        self.closest_hints(
            &ctx.history()
                .iter()
                .rev()
                .filter(|h| self.command(h).is_none())
                .filter(|h| !ctx.history().iter().find(|a| a == h).is_none())
                .map(|h| h.trim())
                .collect::<Vec<_>>(),
            hargs,
        )
        .iter()
        .copied()
        .map(str::to_owned)
        .collect()
    }

    fn close_command_hints(&self, hargs: &HintArgs) -> Vec<String> {
        self.closest_hints(
            &self.commands.iter().map(Command::name).collect::<Vec<_>>(),
            hargs,
        )
        .iter()
        .copied()
        .map(str::to_owned)
        .collect()
    }

    pub(crate) fn file_hints(&self, hargs: &HintArgs) -> Vec<String> {
        self.completer
            .complete(&hargs.line, hargs.pos, unsafe {
                NonNull::dangling().as_ref()
            })
            .map(|(_, p)| p)
            .unwrap_or_default()
            .iter()
            .map(Pair::display)
            .map(str::to_owned)
            .collect()
    }

    pub(crate) fn subcmd_hints(&self, hargs: &HintArgs, subcmd: &Command) -> Vec<String> {
        // TODO: recursive subcmds based on `hargs.subcmd_index`
        self.closest_hints(
            &subcmd.names.iter().map(String::as_str).collect::<Vec<_>>(),
            hargs,
        )
        .iter()
        .copied()
        .map(str::to_owned)
        .collect()
    }
}

impl Completer for MyHelper<'_> {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        if pos < line.len() {
            return Ok((0, vec![]));
        }

        let mut hints = vec![];
        let hargs = HintArgs {
            line: line.to_owned(),
            pos,
        };

        if line.is_empty() {
            hints.push("".to_owned());
        }

        let history = self.history_hints(&hargs, ctx);
        if history.len() != 0 {
            hints.extend(history)
        }

        hints.extend_from_slice(
            match self.command(line) {
                Some(cmd) => cmd
                    .args()
                    .get(hargs.subcmd_index().saturating_sub(1))
                    .map_or(vec![], |a| a.hints(hargs.clone(), self)),
                None => self.close_command_hints(&hargs),
            }
            .as_slice(),
        );

        Ok(hargs.hints(hints.iter().collect::<Vec<_>>().as_slice()))
    }
}

impl Hinter for MyHelper<'_> {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.complete(line, pos, ctx)
            .ok()
            .and_then(|(_, p)| p.get(0).and_then(|p| Some(p.replacement.clone())))

        // let hargs = HintArgs { line, pos };
        // self.hinter
        //     .hint(line, pos, ctx)
        //     .or(self.closest_command(&hargs))
    }
}

impl Highlighter for MyHelper<'_> {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Owned(format!("\x1b[1;32m{}\x1b[0m", prompt))
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[38;5;8m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, _pos: usize) -> Cow<'l, str> {
        Borrowed(line)
    }

    fn highlight_char(&self, _line: &str, _pos: usize) -> bool {
        false
    }
}

impl Validator for MyHelper<'_> {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}
