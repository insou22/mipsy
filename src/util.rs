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
