use crate::{
    misc::{
        parse_ident,
        comment_multispace0,
        comment_multispace1,
    },
    number::{
        parse_i8,
        parse_i16,
        parse_i32,
        parse_f32,
        parse_f64,
    }
};
use nom::{
    IResult,
    sequence::tuple,
    combinator::{
        map,
        opt,
    },
    branch::alt,
    multi::separated_list0,
    character::complete::{
        char,
        none_of,
        one_of,
        space0,
    },
    bytes::complete::{
        tag,
        escaped,
    },
    number::complete::le_u32,
};

#[derive(Debug, Clone)]
pub enum Directive {
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

pub fn parse_directive(i: &[u8]) -> IResult<&[u8], Directive> {
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

fn parse_text(i: &[u8]) -> IResult<&[u8], Directive> {
    let (
        remaining_data,
        ..
    ) = tag(".text")(i)?;

    Ok((remaining_data, Directive::Text))
}

fn parse_data(i: &[u8]) -> IResult<&[u8], Directive> {
    let (
        remaining_data,
        ..
    ) = tag(".data")(i)?;

    Ok((remaining_data, Directive::Text))
}

fn parse_ascii_type(tag_str: &'static str) -> impl FnMut(&[u8]) -> IResult<&[u8], String> {
    move |i| {
        let (
            remaining_data,
            (
                _,
                _,
                _,
                text,
                ..
            )
        ) = tuple((
            tag(tag_str),
            space0,
            char('"'),
            escaped(
                none_of("\\\""),
                '\\',
                one_of(r#"0rnt\"'"#),
            ),
            char('"'),
        ))(i)?;

        let text = String::from_utf8(text.into()).unwrap();

        Ok((remaining_data, text))
    }
}

fn parse_ascii(i: &[u8]) -> IResult<&[u8], Directive> {
    map(
        parse_ascii_type(".ascii"),
        |text| Directive::Ascii(text)
    )(i)
}

fn parse_asciiz(i: &[u8]) -> IResult<&[u8], Directive> {
    map(
        parse_ascii_type(".asciiz"),
        |text| Directive::Asciiz(text)
    )(i)
}

fn parse_num_type<T>(tag_str: &'static str, parser: fn(&[u8]) -> IResult<&[u8], T>) -> impl FnMut(&[u8]) -> IResult<&[u8], Vec<T>>
{
    move |i| {
        let (
            remaining_data,
            (
                _,
                list,
            )
        ): (&[u8], (_, Vec<T>)) 
        = tuple((
            tag(tag_str),
            separated_list0(
                tuple((
                    comment_multispace0,
                    opt(
                        tuple((
                            char(','),
                            comment_multispace0,
                        ))
                    ),
                )),
                parser,
            ),
        ))(i)?;

        Ok((remaining_data, list))
    }
}

fn parse_byte(i: &[u8]) -> IResult<&[u8], Directive> {
    map(
        parse_num_type(".byte", parse_i8),
        |data| Directive::Byte(data),
    )(i)
}

fn parse_half(i: &[u8]) -> IResult<&[u8], Directive> {
    map(
        parse_num_type(".half", parse_i16),
        |data| Directive::Half(data),
    )(i)
}

fn parse_word(i: &[u8]) -> IResult<&[u8], Directive> {
    map(
        parse_num_type(".word", parse_i32),
        |data| Directive::Word(data),
    )(i)
}

fn parse_float(i: &[u8]) -> IResult<&[u8], Directive> {
    map(
        parse_num_type(".float", parse_f32),
        |data| Directive::Float(data),
    )(i)
}

fn parse_double(i: &[u8]) -> IResult<&[u8], Directive> {
    map(
        parse_num_type(".double", parse_f64),
        |data| Directive::Double(data),
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
            le_u32,
        ))(i)?;

        Ok((remaining_data, num))
    }
}

fn parse_space(i: &[u8]) -> IResult<&[u8], Directive> {
    map(
        parse_u32_type(".space"),
        |num| Directive::Space(num),
    )(i)
}

fn parse_align(i: &[u8]) -> IResult<&[u8], Directive> {
    map(
        parse_u32_type(".align"),
        |num| Directive::Space(num),
    )(i)
}

fn parse_globl(i: &[u8]) -> IResult<&[u8], Directive> {
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

    Ok((remaining_data, Directive::Globl(ident)))
}