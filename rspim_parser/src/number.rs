use crate::misc::{
    parse_escaped_char,
    parse_ident,
};
use nom::{
    IResult,
    branch::alt,
    combinator::map,
    sequence::tuple,
    character::complete::char,
    number::complete::{
        le_i8, le_i16, le_i32, le_f32, le_f64
    },
};

#[derive(Clone)]
pub enum Number {
    Immediate(Immediate),
    Float32(f32),
    Float64(f64),
    Char(char),
}

#[derive(Clone)]
pub enum Immediate {
    I16(i16),
    I32(i32),
    LabelReference(String),
}

pub fn parse_number(i: &[u8]) -> IResult<&[u8], Number> {
    alt((
        map(parse_immediate, |i| Number::Immediate(i)),
        map(parse_f32,       |f| Number::Float32(f)),
        map(parse_f64,       |f| Number::Float64(f)),
        map(parse_char,      |c| Number::Char(c)),
    ))(i)
}

pub fn parse_immediate(i: &[u8]) -> IResult<&[u8], Immediate> {
    alt((
        map(parse_i16,      |i| Immediate::I16(i)),
        map(parse_i32,      |i| Immediate::I32(i)),
        map(parse_labelref, |l| Immediate::LabelReference(l)),
    ))(i)
}

pub fn parse_i8(i: &[u8]) -> IResult<&[u8], i8> {
    le_i8(i)
}

pub fn parse_i16(i: &[u8]) -> IResult<&[u8], i16> {
    le_i16(i)
}

pub fn parse_i32(i: &[u8]) -> IResult<&[u8], i32> {
    le_i32(i)
}

pub fn parse_labelref(i: &[u8]) -> IResult<&[u8], String> {
    parse_ident(i)
}

pub fn parse_f32(i: &[u8]) -> IResult<&[u8], f32> {
    le_f32(i)
}

pub fn parse_f64(i: &[u8]) -> IResult<&[u8], f64> {
    le_f64(i)
}

fn parse_char(i: &[u8]) -> IResult<&[u8], char> {
    let (
        remaining_data,
        (
            _,
            chr,
            _,
        )
    ) = tuple((
        char('\''),
        parse_escaped_char,
        char('\''),
    ))(i)?;

    Ok((remaining_data, chr as char))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char() {
        let mut chars = String::new();
        chars.push_str(&('A'..='Z').collect::<String>());
        chars.push_str(&('a'..='z').collect::<String>());
        chars.push_str(&('0'..='9').collect::<String>());
        chars.push_str(&"`~!@#$%^&*()-_=+[{]}|;:,<.>/?");

        for chr in chars.chars() {
            assert_eq!(parse_char(format!("'{}'", chr).as_bytes()), Ok(("".as_bytes(), chr)));
        }

        let mut escaped = std::collections::HashMap::<char, char>::new();
        escaped.insert('0',  '\0');
        escaped.insert('r',  '\r');
        escaped.insert('n',  '\n');
        escaped.insert('t',  '\t');
        escaped.insert('\\', '\\');
        escaped.insert('\"', '\"');
        escaped.insert('\'', '\'');

        for (chr, escaped) in escaped {
            assert_eq!(parse_char(format!("'\\{}'", chr).as_bytes()), Ok(("".as_bytes(), escaped)));
        }
    }
}