pub mod compile_error;
pub mod runtime_error;

pub type RSpimResult<T> = Result<T, RSpimError>;
pub type CompileError   = compile_error::CompileError;
pub type RuntimeError   = runtime_error::RuntimeError;

#[derive(Debug)]
pub enum RSpimError {
    Compile(CompileError),
    Runtime(RuntimeError),
}

#[macro_export]
macro_rules! cerr {
    ($err:expr) => {
        Err(crate::error::RSpimError::Compile($err))
    };
}

#[macro_export]
macro_rules! rerr {
    ($err:expr) => {
        Err(crate::error::RSpimError::Runtime($err))
    };
}
