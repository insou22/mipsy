use std::borrow::Cow::{
    self,
    Borrowed,
    Owned,
};
use rustyline::{
    Context,
    error::ReadlineError,
    completion::{
        Completer, 
        FilenameCompleter, 
        Pair,
    },
    highlight::Highlighter,
    hint::{
        Hinter,
        HistoryHinter,
    },
    validate::{
        Validator,
        ValidationContext,
        ValidationResult
    },
};
use rustyline_derive::Helper;

#[derive(Helper)]
pub(crate) struct MyHelper {
    completer: FilenameCompleter,
    hinter: HistoryHinter,
}

impl MyHelper {
    pub(super) fn new() -> Self {
        Self {
            completer: FilenameCompleter::new(),
            hinter: HistoryHinter {},
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
        self.completer.complete(line, pos, ctx)
    }
}

impl Hinter for MyHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &Context<'_>) -> Option<String> {
        self.hinter.hint(line, pos, ctx)
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
    fn validate(
        &self,
        _ctx: &mut ValidationContext,
    ) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}
