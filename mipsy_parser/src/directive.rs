use std::iter::FromIterator;

use crate::{
    Span,
    misc::{
        parse_ident,
        parse_escaped_char,
        comment_multispace0,
        comment_multispace1,
    },
    number::{
        parse_u32,
        parse_f32,
        parse_f64,
    },
};
use nom::{
    IResult,
    branch::alt,
    sequence::tuple,
    bytes::complete::{
        tag,
    },
    character::complete::char,
    combinator::{
        map,
    },
    multi::{
        many_till,
        separated_list1
    },
};

#[derive(Debug, Clone, PartialEq)]
pub enum MPDirective {
    Text,
    Data,
    KText,
    KData,
    Ascii(String),
    Asciiz(String),
    Byte(Vec<u8>),
    Half(Vec<u16>),
    Word(Vec<u32>),
    Float(Vec<f32>),
    Double(Vec<f64>),
    Align(u32),
    Space(u32),
    Globl(String),
}

pub fn parse_directive<'a>(i: Span<'a>) -> IResult<Span<'a>, MPDirective> {
    alt((
        parse_text,
        parse_data,
        parse_ktext,
        parse_kdata,
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

fn parse_text<'a>(i: Span<'a>) -> IResult<Span<'a>, MPDirective> {
    let (
        remaining_data,
        ..
    ) = tag(".text")(i)?;

    Ok((remaining_data, MPDirective::Text))
}

fn parse_data<'a>(i: Span<'a>) -> IResult<Span<'a>, MPDirective> {
    let (
        remaining_data,
        ..
    ) = tag(".data")(i)?;

    Ok((remaining_data, MPDirective::Data))
}

fn parse_ktext<'a>(i: Span<'a>) -> IResult<Span<'a>, MPDirective> {
    let (
        remaining_data,
        ..
    ) = tag(".ktext")(i)?;

    Ok((remaining_data, MPDirective::KText))
}

fn parse_kdata<'a>(i: Span<'a>) -> IResult<Span<'a>, MPDirective> {
    let (
        remaining_data,
        ..
    ) = tag(".kdata")(i)?;

    Ok((remaining_data, MPDirective::KData))
}

fn parse_ascii_type<'a>(tag_str: &'static str) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, String> {
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
            comment_multispace0,
            char('"'),
            many_till(parse_escaped_char, char('"')),
        ))(i)?;

        let text = String::from_iter(text.iter());

        Ok((remaining_data, text))
    }
}

fn parse_ascii<'a>(i: Span<'a>) -> IResult<Span<'a>, MPDirective> {
    map(
        parse_ascii_type(".ascii"),
        MPDirective::Ascii
    )(i)
}

fn parse_asciiz<'a>(i: Span<'a>) -> IResult<Span<'a>, MPDirective> {
    map(
        parse_ascii_type(".asciiz"),
        MPDirective::Asciiz
    )(i)
}

fn parse_num_type<'a, T>(tag_str: &'static str, parser: fn(Span<'a>) -> IResult<Span<'a>, T>) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Vec<T>>
{
    move |i| {
        let (
            remaining_data,
            (
                _,
                _,
                list,
            )
        ): (Span<'a>, (_, _, Vec<T>)) 
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

fn parse_byte<'a>(i: Span<'a>) -> IResult<Span<'a>, MPDirective> {
    map(
        parse_num_type(".byte", crate::number::parse_byte),
        MPDirective::Byte,
    )(i)
}

fn parse_half<'a>(i: Span<'a>) -> IResult<Span<'a>, MPDirective> {
    map(
        parse_num_type(".half", crate::number::parse_half),
        MPDirective::Half,
    )(i)
}

fn parse_word<'a>(i: Span<'a>) -> IResult<Span<'a>, MPDirective> {
    map(
        parse_num_type(".word", crate::number::parse_word),
        MPDirective::Word,
    )(i)
}

fn parse_float<'a>(i: Span<'a>) -> IResult<Span<'a>, MPDirective> {
    map(
        parse_num_type(".float", parse_f32),
        MPDirective::Float,
    )(i)
}

fn parse_double<'a>(i: Span<'a>) -> IResult<Span<'a>, MPDirective> {
    map(
        parse_num_type(".double", parse_f64),
        MPDirective::Double,
    )(i)
}

fn parse_u32_type<'a>(tag_str: &'static str) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, u32> {
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

fn parse_space<'a>(i: Span<'a>) -> IResult<Span<'a>, MPDirective> {
    map(
        parse_u32_type(".space"),
        MPDirective::Space,
    )(i)
}

fn parse_align<'a>(i: Span<'a>) -> IResult<Span<'a>, MPDirective> {
    map(
        parse_u32_type(".align"),
        MPDirective::Space,
    )(i)
}

fn parse_globl<'a>(i: Span<'a>) -> IResult<Span<'a>, MPDirective> {
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::misc::{span, unspan};

    #[test]
    fn test_ascii() {
        assert_eq!(
            unspan(parse_ascii(span(".ascii \"hello\\n\"")).unwrap()),
            (
                "".to_string(), 
                MPDirective::Ascii("hello\n".to_string())
            )
        );

        assert_eq!(
            unspan(parse_ascii(span(".ascii \"hello \\\"jeff\\\"\"")).unwrap()),
            (
                "".to_string(),
                MPDirective::Ascii("hello \"jeff\"".to_string())
            )
        );
    }
}