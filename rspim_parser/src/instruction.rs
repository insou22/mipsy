use crate::{
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

#[derive(Debug, Clone)]
pub struct MPInstruction {
    name: String,
    arguments: Vec<MPArgument>,
}

#[derive(Debug, Clone)]
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

pub fn parse_instruction(i: &[u8]) -> IResult<&[u8], MPInstruction> {
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

fn parse_argument(i: &[u8]) -> IResult<&[u8], MPArgument> {
    alt((
        parse_argument_reg,
        parse_argument_num,
    ))(i)
}

fn parse_argument_reg(i: &[u8]) -> IResult<&[u8], MPArgument> {
    map(
        parse_register,
        |reg| MPArgument::Register(reg)
    )(i)
}

fn parse_argument_num(i: &[u8]) -> IResult<&[u8], MPArgument> {
    map(
        parse_number,
        |num| MPArgument::Number(num)
    )(i)
}