use rustyline::{
    completion::{Completer, FilenameCompleter, Pair},
    error::ReadlineError,
    highlight::Highlighter,
    hint::{Hinter, HistoryHinter},
    validate::{ValidationContext, ValidationResult, Validator},
    Context,
};
use rustyline_derive::Helper;
use std::borrow::Cow::{self, Borrowed, Owned};

#[derive(Helper)]
pub(crate) struct MyHelper {
    completer: FilenameCompleter,
    hinter: HistoryHinter,
    defaults: Vec<String>,
}

impl MyHelper {
    pub(super) fn new() -> Self {
        Self {
            completer: FilenameCompleter::new(),
            hinter: HistoryHinter {},
            defaults: vec![],
        }
    }

    pub(super) fn set_defaults(&mut self, defaults: Vec<String>) {
        self.defaults = defaults
    }

    fn closest<'a>(&self, with: &Vec<&'a str>, line: &str, pos: usize) -> Vec<&'a str> {
        if line.is_empty() || pos < line.len() {
            vec![]
        } else {
            with.iter()
                .filter(|m| m.len() != pos)
                .filter(|m| m.starts_with(line))
                .map(|m| *m)
                .collect()
        }
    }

    fn closest_default(&self, line: &str, pos: usize) -> Option<String> {
        if line.is_empty() || pos < line.len() {
            None
        } else if let Some(found) = self.defaults.iter().find(|s| s.starts_with(line)) {
            if found.len() == pos {
                None
            } else {
                Some(found[pos..].to_owned())
            }
        } else {
            None
        }
    }
}

impl Completer for MyHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.completer.complete(line, pos, ctx).and_then(|x| {
            // will be 0 for first arg. kinda a hacky way to check if we should match
            // against files or not but thats okay.
            // TODO: something like `match self.arg_type() { label => , file => number => }`
            Ok(if x.0 == 0 {
                (
                    pos,
                    self.closest(
                        &[
                            ctx.history()
                                .iter()
                                .rev()
                                .filter(|h| !self.defaults.contains(h))
                                .map(|h| &h[..])
                                .collect::<Vec<&str>>(),
                            self.defaults.iter().map(|s| &s[..]).collect(),
                        ]
                        .concat(),
                        line,
                        pos,
                    )
                    .iter()
                    .map(|m| Pair {
                        display: m.to_string(),
                        replacement: m[pos..].to_owned(),
                    })
                    .collect(),
                )
            } else {
                x
            })
        })
    }
}

impl Hinter for MyHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter
            .hint(line, pos, ctx)
            .or(self.closest_default(line, pos))
    }
}

impl Highlighter for MyHelper {
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

impl Validator for MyHelper {
    fn validate(&self, _ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}
