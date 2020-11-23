use std::fmt;

use crate::{
    Span,
    number::{
        MPImmediate,
        parse_immediate,
    },
};
use nom::{
    IResult,
    sequence::tuple,
    branch::alt,
    combinator::opt,
    character::complete::{
        char,
        digit1,
        alphanumeric1,
        space0,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum MPRegister {
    Normal(MPRegisterIdentifier),
    Offset(MPImmediate, MPRegisterIdentifier),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MPRegisterIdentifier {
    Numbered(u8),
    Named(String),
}

impl MPRegister {
    fn get_identifier(&self) -> &MPRegisterIdentifier {
        match self {
            Self::Normal(ident) => ident,
            Self::Offset(_, ident) => ident,
        }
    }
}

impl fmt::Display for MPRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal(id)      => write!(f, "${}", id),
            Self::Offset(imm, id) => write!(f, "{}(${})", imm, id),
        }
    }
}

impl fmt::Display for MPRegisterIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Numbered(num) => write!(f, "{}", num),
            Self::Named(name)   => write!(f, "{}", name),
        }
    }
}


pub fn parse_register<'a>(i: Span<'a>) -> IResult<Span<'a>, MPRegister> {
    alt((
        parse_normal_register,
        parse_offset_register,
    ))(i)
}

pub fn parse_normal_register<'a>(i: Span<'a>) -> IResult<Span<'a>, MPRegister> {
    let (
        remaining_data,
        (
            _,
            text,
        ),
    ) = tuple((
        char('$'),
        alt((
            digit1,
            alphanumeric1,
        )),
    ))(i)?;

    let text = String::from_utf8_lossy(text.fragment()).to_string();

    Ok((remaining_data, MPRegister::Normal(
        if let Ok(num) = text.parse::<u8>() {
            MPRegisterIdentifier::Numbered(num)
        } else {
            MPRegisterIdentifier::Named(text)
        }
    )))
}

pub fn parse_offset_register<'a>(i: Span<'a>) -> IResult<Span<'a>, MPRegister> {
    let (
        remaining_data,
        (
            imm,
            _,
            _,
            _,
            reg,
            ..,
        )
    ) = tuple((
        opt(parse_immediate),
        space0,
        char('('),
        space0,
        parse_normal_register,
        space0,
        char(')'),
    ))(i)?;

    Ok((
        remaining_data, 
        MPRegister::Offset(
            imm.unwrap_or(MPImmediate::I16(0)), 
            reg.get_identifier().clone()
        )
    ))
}
