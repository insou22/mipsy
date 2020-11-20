use std::fmt;

use nom_locate::position;
use crate::{
    Span,
    register::{
        MPRegister,
        parse_register,
    },
    number::{
        MPNumber,
        parse_number,
    },
    misc::{
        parse_ident,
        comment_multispace0,
    },
};
use nom::{
    IResult,
    sequence::tuple,
    combinator::map,
    branch::alt,
    multi::separated_list0,
    character::complete::{
        char,
        space0,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct MPInstruction {
    pub(crate) name: String,
    pub(crate) arguments: Vec<(MPArgument, u32, u32)>,
    pub(crate) col: u32,
    pub(crate) col_end: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MPArgument {
    Register(MPRegister),
    Number(MPNumber),
}

impl MPInstruction {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn arguments(&self) -> Vec<&(MPArgument, u32, u32)> {
        self.arguments.iter().collect()
    }

    pub fn col(&self) -> u32 {
        self.col
    }

    pub fn col_end(&self) -> u32 {
        self.col_end
    }
}

impl fmt::Display for MPArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Register(reg) => write!(f, "{}", reg),
            Self::Number(num)   => write!(f, "{}", num),
        }
    }
}

pub fn parse_instruction<'a>(i: Span<'a>) -> IResult<Span<'a>, MPInstruction> {
    let (
        remaining_data,
        (
            position,
            name,
            _,
            arguments,
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
        position,
        comment_multispace0,
    ))(i)?;

    Ok((remaining_data, MPInstruction { name, arguments, col: position.get_column() as u32, col_end: position_end.get_column() as u32 }))
}

pub fn parse_argument<'a>(i: Span<'a>) -> IResult<Span<'a>, (MPArgument, u32, u32)> {
    map(
        tuple((
            position,
            alt((
                parse_argument_reg,
                parse_argument_num,
            )),
            position,
        )),
        |(pos, arg, pos_end)| (arg, pos.get_column() as u32, pos_end.get_column() as u32)
    )(i)
}

fn parse_argument_reg<'a>(i: Span<'a>) -> IResult<Span<'a>, MPArgument> {
    map(
        parse_register,
        |reg| MPArgument::Register(reg)
    )(i)
}

fn parse_argument_num<'a>(i: Span<'a>) -> IResult<Span<'a>, MPArgument> {
    map(
        parse_number,
        |num| MPArgument::Number(num)
    )(i)
}