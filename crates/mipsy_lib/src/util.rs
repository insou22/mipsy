use serde::{Deserialize, Serialize};

#[derive(Copy, Debug, Serialize, Deserialize)]
pub enum Safe<T> {
    Valid(T),
    Uninitialised,
}

impl<T> Safe<T> {
    pub fn valid(value: T) -> Self {
        Safe::Valid(value)
    }

    pub fn into_option(self) -> Option<T> {
        match self {
            Self::Valid(t) => Some(t),
            Self::Uninitialised => None,
        }
    }

    pub fn as_option(&self) -> Option<&T> {
        match self {
            Self::Valid(t) => Some(t),
            Self::Uninitialised => None,
        }
    }
}

impl<T> PartialEq for Safe<T>
where
    T: PartialEq
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Valid(a), Self::Valid(b)) => {
                a.eq(b)
            }
            (Self::Uninitialised, Self::Uninitialised) => true,
            _ => false,
        }
    }
}

impl<T> Clone for Safe<T>
where 
    T: Clone
{
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
