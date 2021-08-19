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
pub enum MpDirective {
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

pub fn parse_directive(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
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

fn parse_text(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    let (
        remaining_data,
        ..
    ) = tag(".text")(i)?;

    Ok((remaining_data, MpDirective::Text))
}

fn parse_data(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    let (
        remaining_data,
        ..
    ) = tag(".data")(i)?;

    Ok((remaining_data, MpDirective::Data))
}

fn parse_ktext(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    let (
        remaining_data,
        ..
    ) = tag(".ktext")(i)?;

    Ok((remaining_data, MpDirective::KText))
}

fn parse_kdata(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    let (
        remaining_data,
        ..
    ) = tag(".kdata")(i)?;

    Ok((remaining_data, MpDirective::KData))
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

        let text = text.iter().collect();

        Ok((remaining_data, text))
    }
}

fn parse_ascii(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    map(
        parse_ascii_type(".ascii"),
        MpDirective::Ascii
    )(i)
}

fn parse_asciiz(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    map(
        parse_ascii_type(".asciiz"),
        MpDirective::Asciiz
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

fn parse_byte(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    map(
        parse_num_type(".byte", crate::number::parse_byte),
        MpDirective::Byte,
    )(i)
}

fn parse_half(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    map(
        parse_num_type(".half", crate::number::parse_half),
        MpDirective::Half,
    )(i)
}

fn parse_word(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    map(
        parse_num_type(".word", crate::number::parse_word),
        MpDirective::Word,
    )(i)
}

fn parse_float(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    map(
        parse_num_type(".float", parse_f32),
        MpDirective::Float,
    )(i)
}

fn parse_double(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    map(
        parse_num_type(".double", parse_f64),
        MpDirective::Double,
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

fn parse_space(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    map(
        parse_u32_type(".space"),
        MpDirective::Space,
    )(i)
}

fn parse_align(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    map(
        parse_u32_type(".align"),
        MpDirective::Space,
    )(i)
}

fn parse_globl(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
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

    Ok((remaining_data, MpDirective::Globl(ident)))
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
                MpDirective::Ascii("hello\n".to_string())
            )
        );

        assert_eq!(
            unspan(parse_ascii(span(".ascii \"hello \\\"jeff\\\"\"")).unwrap()),
            (
                "".to_string(),
                MpDirective::Ascii("hello \"jeff\"".to_string())
            )
        );
    }
}