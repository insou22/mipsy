use crate::error::{
    MipsyResult,
    MipsyError,
    CompileError,
};

pub(crate) fn cerr<T>(error: CompileError) -> MipsyResult<T> {
    Err(MipsyError::Compile(error))
}

pub(crate) trait WithLoc<T>
where
    Self: Sized
{
    fn with_line(self, line: u32) -> MipsyResult<T>;
    fn with_col(self, col: u32) -> MipsyResult<T>;
    fn with_col_end(self, col_end: u32) -> MipsyResult<T>;
}

impl<T> WithLoc<T> for MipsyResult<T> {
    fn with_line(self, line: u32) -> MipsyResult<T> {
        self.map_err(|err| {
            match err {
                MipsyError::Compile(error) => MipsyError::CompileLoc { line: Some(line), col: None, col_end: None, error },
                MipsyError::CompileLoc { line: None, col, col_end, error } => MipsyError::CompileLoc { line: Some(line), col, col_end, error },
                MipsyError::CompileLoc { line: Some(x), col, col_end, error } => MipsyError::CompileLoc { line: Some(x), col, col_end, error },
                _ => err
            }
        })
    }

    fn with_col(self, col: u32) -> MipsyResult<T> {
        self.map_err(|err| {
            match err {
                MipsyError::Compile(error) => MipsyError::CompileLoc { line: None, col: Some(col), col_end: None, error },
                MipsyError::CompileLoc { line, col: None, col_end, error } => MipsyError::CompileLoc { line, col: Some(col), col_end, error },
                MipsyError::CompileLoc { line, col: Some(x), col_end, error } => MipsyError::CompileLoc { line, col: Some(x), col_end, error },
                _ => err
            }
        })
    }

    fn with_col_end(self, col_end: u32) -> MipsyResult<T> {
        self.map_err(|err| {
            match err {
                MipsyError::Compile(error) => MipsyError::CompileLoc { line: None, col: None, col_end: Some(col_end), error },
                MipsyError::CompileLoc { line, col, col_end: None, error } => MipsyError::CompileLoc { line, col, col_end: Some(col_end), error },
                MipsyError::CompileLoc { line, col, col_end: Some(x), error } => MipsyError::CompileLoc { line, col, col_end: Some(x), error },
                _ => err
            }
        })
    }
}

#[derive(Copy, Debug)]
pub enum Safe<T> {
    Valid(T),
    Uninitialised,
}

impl<T> Safe<T> {
    pub fn valid(value: T) -> Self {
        Safe::Valid(value)
    }
}

impl<T> Clone for Safe<T>
where T: Clone {
    fn clone(&self) -> Self {
        match self {
            Self::Valid(t) => Self::Valid(t.clone()),
            Self::Uninitialised => Self::Uninitialised,
        }
    }
}

impl<T> Default for Safe<T> {
    fn default() -> Self {
        Self::Uninitialised
    }
}

pub trait TruncImm {
    fn trunc_imm(&self) -> Self;
}

impl TruncImm for i32 {
    fn trunc_imm(&self) -> Self {
        *self as i16 as Self
    }
}

impl TruncImm for u32 {
    fn trunc_imm(&self) -> Self {
        *self as i16 as Self
    }
}
