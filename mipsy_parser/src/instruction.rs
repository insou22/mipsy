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
    pub(crate) arguments: Vec<MPArgument>,
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

    pub fn arguments(&self) -> Vec<&MPArgument> {
        self.arguments.iter().collect()
    }
}

pub fn parse_instruction<'a>(i: Span<'a>) -> IResult<Span<'a>, MPInstruction> {
    let (
        remaining_data,
        (
            name,
            _,
            arguments,
            ..
        )
    ) = tuple((
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
        comment_multispace0,
    ))(i)?;

    Ok((remaining_data, MPInstruction { name, arguments }))
}

pub fn parse_argument<'a>(i: Span<'a>) -> IResult<Span<'a>, MPArgument> {
    alt((
        parse_argument_reg,
        parse_argument_num,
    ))(i)
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