use serde::{Deserialize, Serialize};
use std::fmt;

use crate::{
    number::{parse_immediate, MpImmediate},
    Span,
};
use nom::{IResult, branch::alt, character::complete::{alphanumeric1, char, digit1, one_of, space0}, combinator::{map, opt}, sequence::tuple};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MpRegister {
    Normal(MpRegisterIdentifier),
    Offset(MpImmediate, MpRegisterIdentifier),
    BinaryOpOffset(MpImmediate, MpOffsetOperator, MpImmediate, MpRegisterIdentifier),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MpRegisterIdentifier {
    Numbered(u8),
    Named(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MpOffsetOperator {
    Plus,
    Minus,
}

impl MpRegister {
    pub fn get_identifier(&self) -> &MpRegisterIdentifier {
        match self {
            Self::Normal(ident) => ident,
            Self::Offset(_, ident) => ident,
            Self::BinaryOpOffset(_, _, _, ident) => ident,
        }
    }
}

impl fmt::Display for MpRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal(id) => write!(f, "${}", id),
            Self::Offset(imm, id) => write!(f, "{}(${})", imm, id),
            Self::BinaryOpOffset(i1, op, i2, id) => write!(f, "{} {} {}(${})", i1, op, i2, id),
        }
    }
}

impl fmt::Display for MpRegisterIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Numbered(num) => write!(f, "{}", num),
            Self::Named(name) => write!(f, "{}", name),
        }
    }
}

impl fmt::Display for MpOffsetOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Plus  => write!(f, "+"),
            Self::Minus => write!(f, "-"),
        }
    }
}

pub fn parse_register(i: Span<'_>) -> IResult<Span<'_>, MpRegister> {
    alt((parse_normal_register, parse_offset_register, parse_offset_binary_op_register))(i)
}

pub fn parse_normal_register(i: Span<'_>) -> IResult<Span<'_>, MpRegister> {
    let (remaining_data, (_, text)) = tuple((char('$'), alt((digit1, alphanumeric1))))(i)?;

    let text = String::from_utf8_lossy(text.fragment()).to_string();

    Ok((
        remaining_data,
        MpRegister::Normal(if let Ok(num) = text.parse::<u8>() {
            MpRegisterIdentifier::Numbered(num)
        } else {
            MpRegisterIdentifier::Named(text)
        }),
    ))
}

pub fn parse_offset_register(i: Span<'_>) -> IResult<Span<'_>, MpRegister> {
    let (remaining_data, (imm, _, _, _, reg, ..)) = tuple((
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
        MpRegister::Offset(
            imm.unwrap_or(MpImmediate::I16(0)),
            reg.get_identifier().clone(),
        ),
    ))
}

pub fn parse_offset_binary_op_register(i: Span<'_>) -> IResult<Span<'_>, MpRegister> {
    let (remaining_data, (i1, _, op, _, i2, _, _, _, reg, ..)) = tuple((
        parse_immediate,
        space0,
        map(
            one_of("+-"),
            |char| match char {
                '+' => MpOffsetOperator::Plus,
                '-' => MpOffsetOperator::Minus,
                _ => unreachable!(),
            },
        ),
        space0,
        parse_immediate,
        space0,
        char('('),
        space0,
        parse_normal_register,
        space0,
        char(')'),
    ))(i)?;

    Ok((
        remaining_data,
        MpRegister::BinaryOpOffset(i1, op, i2, reg.get_identifier().clone()),
    ))
}
