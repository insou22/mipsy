use mipsy_lib::MipsyError;

pub type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug)]
#[allow(dead_code)]
pub enum CommandError {
    ArgExpectedI32 { arg: String, instead: String, },
    ArgExpectedU32 { arg: String, instead: String, },
    HelpUnknownCommand { command: String },
    CannotReadFile { path: String, os_error: String, },
    CannotCompile  { path: String, program: String, mipsy_error: MipsyError },

    MustLoadFile,
    ProgramExited,

    CannotStepFurtherBack,
    RuntimeError { mipsy_error: MipsyError },

    WithTip { error: Box<CommandError>, tip: String },
}