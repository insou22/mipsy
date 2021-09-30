use std::fmt;

use crate::{
    misc::{comment_multispace0, parse_ident},
    number::{parse_number, MpNumber},
    register::{parse_register, MpRegister},
    Span,
};
use nom::{IResult, branch::alt, character::complete::{
        char,
        space0,
    }, combinator::{map, opt}, multi::separated_list0, sequence::tuple};
use nom_locate::position;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MpInstruction {
    pub(crate) name: String,
    pub(crate) arguments: Vec<(MpArgument, u32, u32)>,
    pub(crate) col: u32,
    pub(crate) col_end: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MpArgument {
    Register(MpRegister),
    Number(MpNumber),
}

impl MpInstruction {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> &[(MpArgument, u32, u32)] {
        &self.arguments
    }

    pub fn arguments_mut(&mut self) -> &mut Vec<(MpArgument, u32, u32)> {
        &mut self.arguments
    }

    pub fn col(&self) -> u32 {
        self.col
    }

    pub fn col_end(&self) -> u32 {
        self.col_end
    }
}

impl fmt::Display for MpArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Register(reg) => write!(f, "{}", reg),
            Self::Number(num) => write!(f, "{}", num),
        }
    }
}

pub fn parse_instruction(i: Span<'_>) -> IResult<Span<'_>, MpInstruction> {
    let (
        remaining_data,
        (
            position,
            name,
            _,
            arguments,
            _,
            position_end,
            ..
        )
    ) = tuple((
        position,
        parse_ident,
        space0,
        separated_list0(
            tuple((
                space0,
                char(','),
                space0,
            )),
            parse_argument,
        ),
        opt(
            tuple((
                comment_multispace0,
                char(';'),
            )),
        ),
        position,
        comment_multispace0,
    ))(i)?;

    Ok((
        remaining_data,
        MpInstruction {
            name,
            arguments,
            col: position.get_column() as u32,
            col_end: position_end.get_column() as u32,
        },
    ))
}

pub fn parse_argument(i: Span<'_>) -> IResult<Span<'_>, (MpArgument, u32, u32)> {
    map(
        tuple((
            position,
            alt((parse_argument_reg, parse_argument_num)),
            position,
        )),
        |(pos, arg, pos_end)| (arg, pos.get_column() as u32, pos_end.get_column() as u32),
    )(i)
}

fn parse_argument_reg(i: Span<'_>) -> IResult<Span<'_>, MpArgument> {
    map(parse_register, MpArgument::Register)(i)
}

fn parse_argument_num(i: Span<'_>) -> IResult<Span<'_>, MpArgument> {
    map(parse_number, MpArgument::Number)(i)
}
