use std::fmt;

use crate::{
    Span,
    misc::{
        escape_char,
        parse_escaped_char,
        parse_ident
    }
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
        is_a,
    },
    number::complete::{
        float,
        double,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum MpNumber {
    Immediate(MpImmediate),
    Float32(f32),
    Float64(f64),
    Char(char),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MpImmediate {
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    LabelReference(String),
}

impl fmt::Display for MpNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Immediate(imm) => write!(f, "{}", imm),
            Self::Float32(float) => write!(f, "{}", float),
            Self::Float64(float) => write!(f, "{}", float),
            Self::Char(char)     => write!(f, "'{}'", escape_char(*char)),
        }
    }
}

impl fmt::Display for MpImmediate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::I16(i) => write!(f, "{}", i),
            Self::U16(i) => write!(f, "{}", i),
            Self::I32(i) => write!(f, "{}", i),
            Self::U32(i) => write!(f, "{}", i),
            Self::LabelReference(label) => write!(f, "{}", label),
        }
    }
}

pub fn parse_number(i: Span<'_>) -> IResult<Span<'_>, MpNumber> {
    alt((
        map(parse_immediate, MpNumber::Immediate),
        map(parse_f32,       MpNumber::Float32),
        map(parse_f64,       MpNumber::Float64),
        map(parse_char,      MpNumber::Char),
    ))(i)
}

pub fn parse_immediate(i: Span<'_>) -> IResult<Span<'_>, MpImmediate> {
    alt((
        map(parse_i16,      MpImmediate::I16),
        map(parse_u16,      MpImmediate::U16),
        map(parse_i32,      MpImmediate::I32),
        map(parse_u32,      MpImmediate::U32),
        map(parse_labelref, MpImmediate::LabelReference),
    ))(i)
}

pub fn parse_num<'a, O: RadixNum<O>>(i: Span<'a>) -> IResult<Span<'a>, O> {
    map_res(
        alt((
            map(
                tuple((
                    opt(char('-')),
                    tag("0x"),
                    hex_digit1,
                )),
                |(neg, _, digits): (Option<char>, _, Span<'a>)| (get_sign(neg), 16, String::from_utf8_lossy(digits.fragment()).to_string())
            ),
            map(
                tuple((
                    opt(char('-')),
                    tag("0b"),
                    is_a("01"),
                )),
                |(neg, _, digits): (Option<char>, _, Span<'a>)| (get_sign(neg), 2, String::from_utf8_lossy(digits.fragment()).to_string())
            ),
            map(
                tuple((
                    opt(char('-')),
                    tag("0o"),
                    oct_digit1,
                )),
                |(neg, _, digits): (Option<char>, _, Span<'a>)| (get_sign(neg), 8, String::from_utf8_lossy(digits.fragment()).to_string())
            ),
            map(
                tuple((
                    opt(char('-')),
                    tag("0"),
                    oct_digit1,
                )),
                |(neg, _, digits): (Option<char>, _, Span<'a>)| (get_sign(neg), 8, String::from_utf8_lossy(digits.fragment()).to_string())
            ),
            map(
                tuple((
                    opt(char('-')),
                    digit1,
                )),
                |(neg, digits): (Option<char>, Span<'a>)| (get_sign(neg), 10, String::from_utf8_lossy(digits.fragment()).to_string())
            ),
            map(
                tuple((
                    tag("'"),
                    parse_escaped_char,
                    tag("'"),
                )),
                |(_, ch, _)| ("", 10, format!("{}", ch as i32))
            ),
        )),
        |(sign, base, digits)| 
            O::from_str_radix(
                &format!(
                    "{}{}",
                    sign,
                    digits
                ),
                base,
            )
    )(i)
}

pub fn parse_byte(i: Span<'_>) -> IResult<Span<'_>, u8> {
    alt((
        parse_u8,
        map(
            parse_i8,
            |byte| byte as u8
        )
    ))(i)
}

pub fn parse_i8(i: Span<'_>) -> IResult<Span<'_>, i8> {
    parse_num(i)
}

pub fn parse_u8(i: Span<'_>) -> IResult<Span<'_>, u8> {
    parse_num(i)
}

pub fn parse_half(i: Span<'_>) -> IResult<Span<'_>, u16> {
    alt((
        parse_u16,
        map(
            parse_i16,
            |half| half as u16
        )
    ))(i)
}

pub fn parse_i16(i: Span<'_>) -> IResult<Span<'_>, i16> {
    parse_num(i)
}

pub fn parse_u16(i: Span<'_>) -> IResult<Span<'_>, u16> {
    parse_num(i)
}

pub fn parse_word(i: Span<'_>) -> IResult<Span<'_>, u32> {
    alt((
        parse_u32,
        map(
            parse_i32,
            |word| word as u32
        )
    ))(i)
}

pub fn parse_i32(i: Span<'_>) -> IResult<Span<'_>, i32> {
    parse_num(i)
}

pub fn parse_u32(i: Span<'_>) -> IResult<Span<'_>, u32> {
    parse_num(i)
}

pub fn parse_labelref(i: Span<'_>) -> IResult<Span<'_>, String> {
    parse_ident(i)
}

pub fn parse_f32(i: Span<'_>) -> IResult<Span<'_>, f32> {
    float(i)
}

pub fn parse_f64(i: Span<'_>) -> IResult<Span<'_>, f64> {
    double(i)
}

pub fn parse_char(i: Span<'_>) -> IResult<Span<'_>, char> {
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
    use crate::misc::{span, unspan};

    #[test]
    fn char() {
        let mut chars = String::new();
        chars.push_str(&('A'..='Z').collect::<String>());
        chars.push_str(&('a'..='z').collect::<String>());
        chars.push_str(&('0'..='9').collect::<String>());
        chars.push_str(&"`~!@#$%^&*()-_=+[{]}|;:,<.>/?");

        for chr in chars.chars() {
            assert_eq!(
                unspan(parse_char(span(&format!("'{}'", chr))).unwrap()), 
                ("".to_string(), chr)
            );
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
            assert_eq!(
                unspan(parse_char(span(&format!("'\\{}'", chr))).unwrap()),
                ("".to_string(), escaped)
            );
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

impl RadixNum<Self> for u8 {
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, std::num::ParseIntError> {
        Self::from_str_radix(src, radix)
    }
}

impl RadixNum<Self> for i16 {
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, std::num::ParseIntError> {
        Self::from_str_radix(src, radix)
    }
}

impl RadixNum<Self> for u16 {
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
