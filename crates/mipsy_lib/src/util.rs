use serde::{Deserialize, Serialize};
use crate::{
    TEXT_TOP,
    TEXT_BOT,
    GLOBAL_BOT,
    STACK_BOT,
    STACK_TOP,
    KTEXT_BOT,
    KDATA_BOT,
};

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

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Segment {
    None,
    Text,
    Data,
    Stack,
    KText,
    KData,
}

pub fn get_segment(address: u32) -> Segment {
    match address {
        // TODO(zkol): Update this when exclusive range matching is stabilised
        _ if address < TEXT_BOT => Segment::None,
        _ if (TEXT_BOT..=TEXT_TOP)  .contains(&address) => Segment::Text,
        _ if (GLOBAL_BOT..STACK_BOT).contains(&address) => Segment::Data,
        _ if (STACK_BOT..=STACK_TOP).contains(&address) => Segment::Stack,
        _ if (KTEXT_BOT..KDATA_BOT) .contains(&address) => Segment::KText,
        _ if address >= KDATA_BOT => Segment::KData,
        _ => unreachable!(),
    }
}
