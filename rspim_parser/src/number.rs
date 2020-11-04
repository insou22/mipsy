use crate::misc::{
    parse_escaped_char,
    parse_ident,
};
use nom::{
    IResult,
    branch::alt,
    combinator::{
        map,
        map_res,
        opt,
    },
    sequence::tuple,
    character::complete::{
        char,
        digit1,
        hex_digit1,
        oct_digit1,
    },
    bytes::complete::{
        tag,
    },
    number::complete::{
        float,
        double,
    },
};

#[derive(Debug, Clone)]
pub enum MPNumber {
    Immediate(MPImmediate),
    Float32(f32),
    Float64(f64),
    Char(char),
}

#[derive(Debug, Clone)]
pub enum MPImmediate {
    I16(i16),
    I32(i32),
    LabelReference(String),
}

pub fn parse_number(i: &[u8]) -> IResult<&[u8], MPNumber> {
    alt((
        map(parse_immediate, |i| MPNumber::Immediate(i)),
        map(parse_f32,       |f| MPNumber::Float32(f)),
        map(parse_f64,       |f| MPNumber::Float64(f)),
        map(parse_char,      |c| MPNumber::Char(c)),
    ))(i)
}

pub fn parse_immediate(i: &[u8]) -> IResult<&[u8], MPImmediate> {
    alt((
        map(parse_i16,      |i| MPImmediate::I16(i)),
        map(parse_i32,      |i| MPImmediate::I32(i)),
        map(parse_labelref, |l| MPImmediate::LabelReference(l)),
    ))(i)
}

pub fn parse_num<O: RadixNum<O>>(i: &[u8]) -> IResult<&[u8], O> {
    map_res(
        alt((
            map(
                tuple((
                    opt(char('-')),
                    tag("0x"),
                    hex_digit1,
                )),
                |(neg, _, digits)| (get_sign(neg), 16, digits)
            ),
            map(
                tuple((
                    opt(char('-')),
                    tag("0"),
                    oct_digit1,
                )),
                |(neg, _, digits)| (get_sign(neg), 8, digits)
            ),
            map(
                tuple((
                    opt(char('-')),
                    digit1,
                )),
                |(neg, digits)| (get_sign(neg), 10, digits)
            )
        )),
        |(sign, base, digits)| 
            O::from_str_radix(
                &format!(
                    "{}{}",
                    sign,
                    String::from_utf8((digits as &[u8]).into()).unwrap()
                ),
                base,
            )
    )(i)
}

pub fn parse_i8(i: &[u8]) -> IResult<&[u8], i8> {
    parse_num(i)
}

pub fn parse_i16(i: &[u8]) -> IResult<&[u8], i16> {
    parse_num(i)
}

pub fn parse_i32(i: &[u8]) -> IResult<&[u8], i32> {
    parse_num(i)
}

pub fn parse_u32(i: &[u8]) -> IResult<&[u8], u32> {
    parse_num(i)
}

pub fn parse_labelref(i: &[u8]) -> IResult<&[u8], String> {
    parse_ident(i)
}

pub fn parse_f32(i: &[u8]) -> IResult<&[u8], f32> {
    float(i)
}

pub fn parse_f64(i: &[u8]) -> IResult<&[u8], f64> {
    double(i)
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

fn get_sign(neg: Option<char>) -> &'static str {
    if let Some('-') = neg {
        "-"
    } else {
        ""
    }
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

pub trait RadixNum<O> {
    fn from_str_radix(src: &str, radix: u32) -> Result<O, std::num::ParseIntError>;
}

impl RadixNum<Self> for i8 {
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, std::num::ParseIntError> {
        Self::from_str_radix(src, radix)
    }
}

impl RadixNum<Self> for i16 {
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, std::num::ParseIntError> {
        Self::from_str_radix(src, radix)
    }
}

impl RadixNum<Self> for i32 {
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, std::num::ParseIntError> {
        Self::from_str_radix(src, radix)
    }
}

impl RadixNum<Self> for u32 {
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, std::num::ParseIntError> {
        Self::from_str_radix(src, radix)
    }
}
