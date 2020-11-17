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
pub enum MPNumber {
    Immediate(MPImmediate),
    Float32(f32),
    Float64(f64),
    Char(char),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MPImmediate {
    I16(i16),
    U16(u16),
    I32(i32),
    U32(u32),
    LabelReference(String),
}

impl MPNumber {
    pub fn to_string(&self) -> String {
        match self {
            Self::Immediate(imm) => imm.to_string(),
            Self::Float32(float) => float.to_string(),
            Self::Float64(float) => float.to_string(),
            Self::Char(char)     => format!("\'{}\'", escape_char(*char)),
        }
    }
}

impl MPImmediate {
    pub fn to_string(&self) -> String {
        match self {
            Self::I16(i) => i.to_string(),
            Self::U16(i) => i.to_string(),
            Self::I32(i) => i.to_string(),
            Self::U32(i) => i.to_string(),
            Self::LabelReference(label) => label.clone(),
        }
    }
}

pub fn parse_number<'a>(i: Span<'a>) -> IResult<Span<'a>, MPNumber> {
    alt((
        map(parse_immediate, |i| MPNumber::Immediate(i)),
        map(parse_f32,       |f| MPNumber::Float32(f)),
        map(parse_f64,       |f| MPNumber::Float64(f)),
        map(parse_char,      |c| MPNumber::Char(c)),
    ))(i)
}

pub fn parse_immediate<'a>(i: Span<'a>) -> IResult<Span<'a>, MPImmediate> {
    alt((
        map(parse_i16,      |i| MPImmediate::I16(i)),
        map(parse_u16,      |i| MPImmediate::U16(i)),
        map(parse_i32,      |i| MPImmediate::I32(i)),
        map(parse_u32,      |i| MPImmediate::U32(i)),
        map(parse_labelref, |l| MPImmediate::LabelReference(l)),
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
                |(neg, _, digits): (Option<char>, _, Span<'a>)| (get_sign(neg), 16, digits)
            ),
            map(
                tuple((
                    opt(char('-')),
                    tag("0b"),
                    is_a("01"),
                )),
                |(neg, _, digits): (Option<char>, _, Span<'a>)| (get_sign(neg), 2, digits)
            ),
            map(
                tuple((
                    opt(char('-')),
                    tag("0"),
                    oct_digit1,
                )),
                |(neg, _, digits): (Option<char>, _, Span<'a>)| (get_sign(neg), 8, digits)
            ),
            map(
                tuple((
                    opt(char('-')),
                    digit1,
                )),
                |(neg, digits): (Option<char>, Span<'a>)| (get_sign(neg), 10, digits)
            )
        )),
        |(sign, base, digits)| 
            O::from_str_radix(
                &format!(
                    "{}{}",
                    sign,
                    String::from_utf8_lossy(digits.fragment()).to_string()
                ),
                base,
            )
    )(i)
}

pub fn parse_i8<'a>(i: Span<'a>) -> IResult<Span<'a>, i8> {
    parse_num(i)
}

pub fn parse_i16<'a>(i: Span<'a>) -> IResult<Span<'a>, i16> {
    parse_num(i)
}

pub fn parse_u16<'a>(i: Span<'a>) -> IResult<Span<'a>, u16> {
    parse_num(i)
}

pub fn parse_i32<'a>(i: Span<'a>) -> IResult<Span<'a>, i32> {
    parse_num(i)
}

pub fn parse_u32<'a>(i: Span<'a>) -> IResult<Span<'a>, u32> {
    parse_num(i)
}

pub fn parse_labelref<'a>(i: Span<'a>) -> IResult<Span<'a>, String> {
    parse_ident(i)
}

pub fn parse_f32<'a>(i: Span<'a>) -> IResult<Span<'a>, f32> {
    float(i)
}

pub fn parse_f64<'a>(i: Span<'a>) -> IResult<Span<'a>, f64> {
    double(i)
}

fn parse_char<'a>(i: Span<'a>) -> IResult<Span<'a>, char> {
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
