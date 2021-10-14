use std::fmt::Display;

use crate::{Span, constant::{MpConstValueLoc, parse_constant_value}, misc::{
        parse_ident,
        parse_escaped_char,
        comment_multispace0,
        comment_multispace1,
    }, number::{parse_f32, parse_f64}, parser::Position};
use nom_locate::position;
use serde::{Serialize, Deserialize};
use nom::{IResult, branch::alt, bytes::complete::{
        tag,
    }, character::complete::{char, space0}, combinator::{map, opt}, multi::{
        many_till,
        separated_list1
    }, sequence::tuple};

pub type MpDirectiveLoc = (MpDirective, Position);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MpDirective {
    Text,
    Data,
    KText,
    KData,
    Ascii(String),
    Asciiz(String),
    Byte(Vec<MpConstValueLoc>),
    Half(Vec<MpConstValueLoc>),
    Word(Vec<MpConstValueLoc>),
    ByteN(MpConstValueLoc, MpConstValueLoc),
    HalfN(MpConstValueLoc, MpConstValueLoc),
    WordN(MpConstValueLoc, MpConstValueLoc),
    Float(Vec<f32>),
    FloatN(f32, MpConstValueLoc),
    Double(Vec<f64>),
    DoubleN(f64, MpConstValueLoc),
    Align(MpConstValueLoc),
    Space(MpConstValueLoc),
    Globl(String),
}

impl Display for MpDirective {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MpDirective::*;

        write!(f, "{}", match self {
            Ascii(_)                => "ascii",
            Asciiz(_)               => "asciiz",
            Byte(_)   | ByteN(..)   => "byte",
            Half(_)   | HalfN(..)   => "half",
            Word(_)   | WordN(..)   => "word",
            Float(_)  | FloatN(..)  => "float",
            Double(_) | DoubleN(..) => "double",
            Align(_)                => "align",
            Space(_)                => "space",
            Globl(_)                => "globl",
            Text                    => "text",
            Data                    => "data",
            KText                   => "ktext",
            KData                   => "kdata",
        })
    }
}

pub fn parse_directive(i: Span<'_>) -> IResult<Span<'_>, MpDirectiveLoc> {
    map(
        tuple((
            position,
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
            )),
            position,
        )),
        |(pos_start, directive, pos_end)| (directive, Position::from_positions(pos_start, pos_end))
    )(i)
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

fn parse_num_type<'a, T: Clone>(tag_str: &'static str, parser: fn(Span<'a>) -> IResult<Span<'a>, T>) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Result<Vec<T>, (T, MpConstValueLoc)>>
{
    move |i| {
        let (
            remaining_data,
            (
                _,
                _,
                list,
                _,
            )
        ): (Span<'a>, (_, _, Result<Vec<T>, (T, MpConstValueLoc)>, _)) 
        = tuple((
            tag(tag_str),
            comment_multispace0,
            alt((
                map(
                    tuple((
                        parser,
                        space0,
                        char(':'),
                        space0,
                        parse_constant_value,
                    )),
                    |(value, _, _, _, n)| {
                        Err((value, n))
                    }
                ),
                map(
                    separated_list1(
                        map(
                            tuple((
                                space0,
                                char(','),
                                space0,
                            )),
                            |_| ()
                        ),
                        parser,
                    ),
                    |list| Ok(list)
                )
            )),
            opt(
                char(',')
            )
        ))(i)?;

        Ok((remaining_data, list))
    }
}

fn parse_byte(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    map(
        parse_num_type(".byte", parse_constant_value),
        |bytes| match bytes {
            Ok(bytes)      => MpDirective::Byte(bytes),
            Err((byte, n)) => MpDirective::ByteN(byte, n),
        },
    )(i)
}

fn parse_half(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    map(
        parse_num_type(".half", parse_constant_value),
        |halfs| match halfs {
            Ok(halfs)      => MpDirective::Half(halfs),
            Err((half, n)) => MpDirective::HalfN(half, n),
        },
    )(i)
}

fn parse_word(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    map(
        parse_num_type(".word", parse_constant_value),
        |words| match words {
            Ok(words)      => MpDirective::Word(words),
            Err((word, n)) => MpDirective::WordN(word, n),
        },
    )(i)
}

fn parse_float(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    map(
        parse_num_type(".float", parse_f32),
        |floats| match floats {
            Ok(floats)      => MpDirective::Float(floats),
            Err((float, n)) => MpDirective::FloatN(float, n),
        },
    )(i)
}

fn parse_double(i: Span<'_>) -> IResult<Span<'_>, MpDirective> {
    map(
        parse_num_type(".double", parse_f64),
        |doubles| match doubles {
            Ok(doubles)      => MpDirective::Double(doubles),
            Err((double, n)) => MpDirective::DoubleN(double, n),
        },
    )(i)
}

fn parse_u32_type<'a>(tag_str: &'static str) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, MpConstValueLoc> {
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
            parse_constant_value,
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
        MpDirective::Align,
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