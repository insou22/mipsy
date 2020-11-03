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
            comment_multispace0,
            alt((
                map(parse_label,       |l| Item::Label(l)),
                map(parse_directive,   |d| Item::Directive(d)),
                map(parse_instruction, |i| Item::Instruction(i)),
            )),
            comment_multispace0,
        )),
        |(_, directive, _)| directive 
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_s() {
        println!("{:#?}", parse_mips("

        # add 17 and 25  and print result

        main:                    #  x, y, z in $t0, $t1, $t2,
            li   $t0, 17         # x = 17;

            li   $t1, 25         # y = 25;

            add  $t2, $t1, $t0   # z = x + y

            move $a0, $t2        # printf(\"%d\", a0);
            li   $v0, 1
            syscall

            li   $a0, '\\n'       # printf(\"%c\", '\\n');
            li   $v0, 11
            syscall

            li   $v0, 0          # return 0
            jr   $ra


        "));

        assert!(false);
    }
}