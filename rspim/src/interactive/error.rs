use rspim_lib::RSpimError;

pub type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug)]
pub enum CommandError {
    ArgExpectedI32 { arg: String, instead: String, },
    ArgExpectedU32 { arg: String, instead: String, },
    HelpUnknownCommand { command: String },
    CannotReadFile { path: String, os_error: String, },
    CannotCompile  { path: String, program: String, rspim_error: RSpimError },

    MustLoadFile,
    ProgramExited,

    CannotStepFurtherBack,
    RuntimeError { rspim_error: RSpimError },

    WithTip { error: Box<CommandError>, tip: String },
}