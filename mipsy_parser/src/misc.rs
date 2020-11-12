use crate::Span;
use nom::{
    IResult,
    Err::Error,
    error::{
        ParseError,
        ErrorKind,
    },
    character::complete::{
        char,
        one_of,
        none_of,
        multispace1,
    },
    multi::{
        many0,
        many1
    },
    combinator::{
        map,
        opt,
    },
    sequence::{
        tuple,
        preceded,
    },
    branch::alt,
    bytes::complete::is_a,
};

const IDENT_FIRST_CHAR:  &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_";
const IDENT_CONTD_CHARS: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789.";

pub fn parse_escaped_char<'a>(i: Span<'a>) -> IResult<Span<'a>, char> {
    alt((
        map(
            tuple((
                char('\\'),
                one_of("0rnt\\\"\'"),
            )),
            |(_, chr)| match chr {
                '0'  => '\0',
                'r'  => '\r',
                'n'  => '\n',
                't'  => '\t',
                '\\' => '\\',
                '\"' => '\"',
                '\'' => '\'',
                _    => unreachable!()
            }
        ),
        map(
            parse_any1,
            |byte| byte as char
        ),
    ))(i)
}

pub fn parse_ident<'a>(i: Span<'a>) -> IResult<Span<'a>, String> {
    let (
        remaining_data,
        (
            chr1,
            rem
        )
    ) = tuple((
        one_of(IDENT_FIRST_CHAR),
        opt(is_a(IDENT_CONTD_CHARS)),
    ))(i)?;

    let mut ident = String::new();
    ident.push(chr1);
    if let Some(rem) = rem {
        ident.push_str(&String::from_utf8_lossy(rem.fragment()).to_string());
    }

    Ok((remaining_data, ident))
}

pub fn parse_any1<'a>(i: Span<'a>) -> IResult<Span<'a>, u8> {
    if i.len() > 0 {
        Ok((Span::new(&i.fragment()[1..]), i[0]))
    } else {
        Err(Error(ParseError::from_error_kind(i, ErrorKind::Eof)))
    }
}

pub fn comment_multispace0<'a>(i: Span<'a>) -> IResult<Span<'a>, ()> {
    map(
        opt(comment_multispace1),
        |_| ()
    )(i)
}

pub fn comment_multispace1<'a>(i: Span<'a>) -> IResult<Span<'a>, ()> {
    map(
        many1(
            alt((
                map(
                    tuple((
                        multispace1,
                        opt(
                            tuple((
                                preceded(char('#'), many0(none_of("\n"))),
                                opt(char('\n')),
                            ))
                        ),
                    )),
                    |_| (),
                ),
                map(
                    tuple((
                        preceded(char('#'), many0(none_of("\n"))),
                        opt(char('\n')),
                    )),
                    |_| (),
                ),
            )),
        ),
        |_| ()
    )(i)
}

#[cfg(test)]
pub(crate) fn span<'a>(string: &'a str) -> Span<'a> {
    Span::new(string.as_bytes())
}

#[cfg(test)]
pub(crate) fn unspan<'a, T>(tuple: (Span<'a>, T)) -> (String, T) {
    match tuple {
        (span, t) => (String::from_utf8_lossy(span.fragment()).to_string(), t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_any1() {
        assert_eq!(
            unspan(parse_any1(span("hello")).unwrap()),
            ("ello".to_string(), 'h' as u8)
        );

        assert_eq!(
            unspan(parse_any1(span("h")).unwrap()),
            ("".to_string(), 'h' as u8)
        );
    }

    #[test]
    fn test_parse_ident() {
        assert_eq!(
            unspan(parse_ident(span("i")).unwrap()),
            ("".to_string(), "i".into())
        );

        assert_eq!(
            unspan(parse_ident(span("abc")).unwrap()),
            ("".to_string(), "abc".into())
        );

    }
}