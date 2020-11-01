use crate::compile::context::Context;

pub mod compile_error;
pub mod runtime_error;

pub type RSpimResult<T> = Result<T, RSpimError>;
pub type CompileError   = compile_error::CompileError;
pub type RuntimeError   = runtime_error::RuntimeError;

#[derive(Debug)]
pub enum RSpimError {
    Compile(CompileError),
    CompileContext(CompileError, Context),
    Runtime(RuntimeError),
}

pub fn ok<T>(t: T) -> RSpimResult<T> {
    Ok(t)
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
