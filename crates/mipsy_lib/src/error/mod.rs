use serde::{Deserialize, Serialize};
use std::rc::Rc;

pub mod compiler;
pub mod parser;
pub mod runtime;
pub mod util;

pub type MipsyResult<T> = Result<T, MipsyError>;
pub type ParserError = parser::ParserError;
pub type CompilerError = compiler::CompilerError;
pub type RuntimeError = runtime::RuntimeError;

pub type MipsyInternalResult<T> = Result<T, InternalError>;

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum MipsyError {
    Parser(parser::ParserError),
    Compiler(compiler::CompilerError),
    Runtime(runtime::RuntimeError),
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum InternalError {
    Parser(parser::Error),
    Compiler(compiler::Error),
    Runtime(runtime::Error),
}

pub trait ToMipsyResult<T> {
    fn into_parser_mipsy_result(self, file_tag: Rc<str>, line: u32, col: u32) -> MipsyResult<T>;
    fn into_compiler_mipsy_result(
        self,
        file_tag: Rc<str>,
        line: u32,
        col: u32,
        col_end: u32,
    ) -> MipsyResult<T>;
    fn into_runtime_mipsy_result(self) -> MipsyResult<T>;
}

impl<T> ToMipsyResult<T> for MipsyInternalResult<T> {
    fn into_parser_mipsy_result(self, file_tag: Rc<str>, line: u32, col: u32) -> MipsyResult<T> {
        match self {
            Ok(t) => Ok(t),
            Err(error) => Err(error.into_parser_mipsy_error(file_tag, line, col)),
        }
    }

    fn into_compiler_mipsy_result(
        self,
        file_tag: Rc<str>,
        line: u32,
        col: u32,
        col_end: u32,
    ) -> MipsyResult<T> {
        match self {
            Ok(t) => Ok(t),
            Err(error) => Err(error.into_compiler_mipsy_error(file_tag, line, col, col_end)),
        }
    }

    fn into_runtime_mipsy_result(self) -> MipsyResult<T> {
        match self {
            Ok(t) => Ok(t),
            Err(error) => Err(error.into_runtime_mipsy_error()),
        }
    }
}

impl InternalError {
    pub fn into_parser_mipsy_error(self, file_tag: Rc<str>, line: u32, col: u32) -> MipsyError {
        match self {
            InternalError::Parser(error) => {
                MipsyError::Parser(ParserError::new(error, file_tag, line, col))
            }
            InternalError::Compiler(..) => {
                panic!("expected error type parser didn't match actual error type compiler")
            }
            InternalError::Runtime(error) => MipsyError::Runtime(RuntimeError::new(error)),
        }
    }

    pub fn into_compiler_mipsy_error(
        self,
        file_tag: Rc<str>,
        line: u32,
        col: u32,
        col_end: u32,
    ) -> MipsyError {
        match self {
            InternalError::Parser(error) => {
                MipsyError::Parser(ParserError::new(error, file_tag, line, col))
            }
            InternalError::Compiler(error) => {
                MipsyError::Compiler(CompilerError::new(error, file_tag, line, col, col_end))
            }
            InternalError::Runtime(error) => MipsyError::Runtime(RuntimeError::new(error)),
        }
    }

    pub fn into_runtime_mipsy_error(self) -> MipsyError {
        match self {
            InternalError::Parser(..) => {
                panic!("expected error type runtime didn't match actual error type parser")
            }
            InternalError::Compiler(..) => {
                panic!("expected error type runtime didn't match actual error type compiler")
            }
            InternalError::Runtime(error) => MipsyError::Runtime(RuntimeError::new(error)),
        }
    }
}

#[macro_export]
macro_rules! cerr {
    ($err:expr) => {
        Err($crate::error::MipsyError::Compile($err))
    };
}

#[macro_export]
macro_rules! clerr {
    ($line:expr, $err:expr) => {
        Err($crate::error::MipsyError::CompileLine {
            line: $line,
            error: $err,
        })
    };
}

#[macro_export]
macro_rules! rerr {
    ($err:expr) => {
        Err($crate::error::MipsyError::Runtime($err))
    };
}
