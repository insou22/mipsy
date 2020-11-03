use crate::{
    directive::{
        Directive,
        parse_directive,
    },
    instruction::{
        Instruction,
        parse_instruction,
    },
    label::parse_label,
    misc::comment_multispace0,
};
use nom::{
    IResult,
    sequence::tuple,
    combinator::map,
    multi::many0,
    branch::alt,
};


#[derive(Debug, Clone)]
pub struct Program {
    items: Vec<Item>,
}

#[derive(Debug, Clone)]
pub enum Item {
    Instruction(Instruction),
    Directive(Directive),
    Label(String),
}

pub fn parse_mips_item(i: &[u8]) -> IResult<&[u8], Item> {
    map(
        tuple((
            alt((
                map(parse_instruction, |i| Item::Instruction(i)),
                map(parse_directive,   |d| Item::Directive(d)),
                map(parse_label,       |l| Item::Label(l)),
            )),
            comment_multispace0,
        )),
        |(directive, ..)| directive 
    )(i)
}

pub fn parse_mips_bytes(i: &[u8]) -> IResult<&[u8], Program> {
    let (
        remaining_input,
        items
    ) = many0(parse_mips_item)(i)?;

    Ok((
        remaining_input,
        Program {
            items
        },
    ))
}

pub fn parse_mips<T>(input: T) -> Result<Program, &'static str>
where
    T: AsRef<str>,
{
    match parse_mips_bytes(input.as_ref().trim().as_bytes()) {
        Ok((_, program)) => Ok(program),
        Err(_) => Err("Failed to parse"),
    }
}