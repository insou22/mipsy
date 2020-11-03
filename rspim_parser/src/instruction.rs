use crate::{
    register::{
        Register,
        parse_register,
    },
    number::{
        Number,
        parse_number,
    },
    misc::parse_ident,
};
use nom::{
    IResult,
    sequence::tuple,
    combinator::map,
    branch::alt,
    multi::separated_list0,
    character::complete::{
        space0,
        multispace0,
        char,
    },
};

#[derive(Clone)]
pub struct Instruction {
    name: String,
    arguments: Vec<Argument>,
}

#[derive(Clone)]
pub enum Argument {
    Register(Register),
    Number(Number),
}

pub fn parse_instruction(i: &[u8]) -> IResult<&[u8], Instruction> {
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
        multispace0,
    ))(i)?;

    Ok((remaining_data, Instruction { name, arguments }))
}

fn parse_argument(i: &[u8]) -> IResult<&[u8], Argument> {
    alt((
        parse_argument_reg,
        parse_argument_num,
    ))(i)
}

fn parse_argument_reg(i: &[u8]) -> IResult<&[u8], Argument> {
    map(
        parse_register,
        |reg| Argument::Register(reg)
    )(i)
}

fn parse_argument_num(i: &[u8]) -> IResult<&[u8], Argument> {
    map(
        parse_number,
        |num| Argument::Number(num)
    )(i)
}