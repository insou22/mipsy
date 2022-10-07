use mipsy_lib::{MipsyError, Register};
use mipsy_parser::ErrorLocation;

pub type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug)]
#[allow(dead_code)]
pub enum CommandError {
    BadArgument        { arg: String, instead: String, },
    MissingArguments   { args: Vec<String>, instead: Vec<String> },
    ArgExpectedI32     { arg: String, instead: String, },
    ArgExpectedU32     { arg: String, instead: String, },
    InvalidBpId        { arg: String, },
    HelpUnknownCommand { command: String },
    CannotReadFile     { path: String, os_error: String, },
    CannotCompile      { mipsy_error: MipsyError },
    CannotParseLine    { line: String, error: ErrorLocation },
    CannotCompileLine  { line: String, error: MipsyError },
    LineDoesNotExist   { line_number: u32 },
    UnknownRegister    { register: String },
    UnknownLabel       { label: String },
    UninitialisedRegister { register: Register },
    UninitialisedPrint { addr: u32 },
    UnterminatedString { good_parts: String },

    MustLoadFile,
    MustSpecifyFile,
    ProgramExited,

    CannotStepFurtherBack,
    RanOutOfHistory,
    RuntimeError { mipsy_error: MipsyError },
    ReplRuntimeError { mipsy_error: MipsyError, line: String },

    WithTip { error: Box<CommandError>, tip: String },
}
