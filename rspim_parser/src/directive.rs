use std::iter::FromIterator;

use crate::{misc::{
        parse_ident,
        comment_multispace0,
        comment_multispace1,
    }, number::{
        parse_i8,
        parse_i16,
        parse_i32,
        parse_u32,
        parse_f32,
        parse_f64,
    }, misc::parse_escaped_char};
use nom::{IResult, branch::alt, bytes::complete::{
        tag,
        escaped,
    }, character::complete::{
        char,
        none_of,
        one_of,
        space0,
    }, combinator::{
        map,
        opt,
    }, multi::{many0, many_till, separated_list1}, sequence::tuple};

#[derive(Debug, Clone, PartialEq)]
pub enum MPDirective {
    Text,
    Data,
    Ascii(String),
    Asciiz(String),
    Byte(Vec<i8>),
    Half(Vec<i16>),
    Word(Vec<i32>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    Align(u32),
    Space(u32),
    Globl(String),
}

pub fn parse_directive(i: &[u8]) -> IResult<&[u8], MPDirective> {
    alt((
        parse_text,
        parse_data,
        parse_ascii,
        parse_asciiz,
        parse_byte,
        parse_half,
        parse_word,
        parse_float,
        parse_double,
        parse_space,
        parse_align,
        parse_globl,
    ))(i)
}

fn parse_text(i: &[u8]) -> IResult<&[u8], MPDirective> {
    let (
        remaining_data,
        ..
    ) = tag(".text")(i)?;

    Ok((remaining_data, MPDirective::Text))
}

fn parse_data(i: &[u8]) -> IResult<&[u8], MPDirective> {
    let (
        remaining_data,
        ..
    ) = tag(".data")(i)?;

    Ok((remaining_data, MPDirective::Data))
}

fn parse_ascii_type(tag_str: &'static str) -> impl FnMut(&[u8]) -> IResult<&[u8], String> {
    move |i| {
        let (
            remaining_data,
            (
                _,
                _,
                _,
                (
                    text,
                    _
                ),
                ..
            )
        ) = tuple((
            tag(tag_str),
            space0,
            char('"'),
            many_till(parse_escaped_char, char('"')),
        ))(i)?;

        let text = String::from_iter(text.iter()).to_string();

        Ok((remaining_data, text))
    }
}

fn parse_ascii(i: &[u8]) -> IResult<&[u8], MPDirective> {
    map(
        parse_ascii_type(".ascii"),
        |text| MPDirective::Ascii(text)
    )(i)
}

fn parse_asciiz(i: &[u8]) -> IResult<&[u8], MPDirective> {
    map(
        parse_ascii_type(".asciiz"),
        |text| MPDirective::Asciiz(text)
    )(i)
}

fn parse_num_type<T>(tag_str: &'static str, parser: fn(&[u8]) -> IResult<&[u8], T>) -> impl FnMut(&[u8]) -> IResult<&[u8], Vec<T>>
{
    move |i| {
        let (
            remaining_data,
            (
                _,
                _,
                list,
            )
        ): (&[u8], (_, _, Vec<T>)) 
        = tuple((
            tag(tag_str),
            comment_multispace0,
            separated_list1(
                alt((
                    map(
                        tuple((
                            comment_multispace0,
                            char(','),
                            comment_multispace0,
                        )),
                        |_| ()
                    ),
                    comment_multispace1,
                )),
                parser,
            ),
        ))(i)?;

        Ok((remaining_data, list))
    }
}

fn parse_byte(i: &[u8]) -> IResult<&[u8], MPDirective> {
    map(
        parse_num_type(".byte", parse_i8),
        |data| MPDirective::Byte(data),
    )(i)
}

fn parse_half(i: &[u8]) -> IResult<&[u8], MPDirective> {
    map(
        parse_num_type(".half", parse_i16),
        |data| MPDirective::Half(data),
    )(i)
}

fn parse_word(i: &[u8]) -> IResult<&[u8], MPDirective> {
    map(
        parse_num_type(".word", parse_i32),
        |data| MPDirective::Word(data),
    )(i)
}

fn parse_float(i: &[u8]) -> IResult<&[u8], MPDirective> {
    map(
        parse_num_type(".float", parse_f32),
        |data| MPDirective::Float(data),
    )(i)
}

fn parse_double(i: &[u8]) -> IResult<&[u8], MPDirective> {
    map(
        parse_num_type(".double", parse_f64),
        |data| MPDirective::Double(data),
    )(i)
}

fn parse_u32_type(tag_str: &'static str) -> impl FnMut(&[u8]) -> IResult<&[u8], u32> {
    move |i| {
        let (
            remaining_data,
            (
                _,
                _,
                num,
                ..
            )
        ) = tuple((
            tag(tag_str),
            comment_multispace0,
            parse_u32,
        ))(i)?;

        Ok((remaining_data, num))
    }
}

fn parse_space(i: &[u8]) -> IResult<&[u8], MPDirective> {
    map(
        parse_u32_type(".space"),
        |num| MPDirective::Space(num),
    )(i)
}

fn parse_align(i: &[u8]) -> IResult<&[u8], MPDirective> {
    map(
        parse_u32_type(".align"),
        |num| MPDirective::Space(num),
    )(i)
}

fn parse_globl(i: &[u8]) -> IResult<&[u8], MPDirective> {
    let (
        remaining_data,
        (
            _,
            _,
            ident
        )
    ) = tuple((
        tag(".globl"),
        comment_multispace1,
        parse_ident
    ))(i)?;

    Ok((remaining_data, MPDirective::Globl(ident)))
}

mod test {
    use super::*;
    #[test]
    fn test_ascii() {
        assert_eq!(
            parse_ascii(".ascii \"hello\\n\"".as_bytes()).unwrap(), 
            (
                "".as_bytes(), 
                MPDirective::Ascii("hello\n".to_string())
            )
        );

        assert_eq!(
            parse_ascii(".ascii \"hello \\\"jeff\\\"\"".as_bytes()).unwrap(), 
            (
                "".as_bytes(), 
                MPDirective::Ascii("hello \"jeff\"".to_string())
            )
        );
    }
}