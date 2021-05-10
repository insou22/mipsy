use std::{rc::Rc, u8};

use crate::Span;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::is_a,
    character::complete::{
        anychar,
        char,
        multispace1,
        none_of,
        one_of
    },
    combinator::{
        map,
        opt,
    },
    multi::{
        many0,
        many1
    },
    sequence::{
        tuple,
        preceded,
    },
};

#[derive(Debug)]
pub struct ErrorLocation {
    pub file_name: Option<Rc<str>>,
    pub line: u32,
    pub col:  usize,
}

pub(crate) fn parse_result<'a, T, P>(i: Span<'a>, file_name: Option<Rc<str>>, mut parser: P) -> Result<T, ErrorLocation> 
where
    P: FnMut(Span<'a>) -> IResult<Span<'a>, T>
{
    match (parser)(i) {
        Ok((leftover, t)) => {
            if leftover.is_empty() {
                Ok(t)
            } else {
                match comment_multispace0(leftover) {
                    Ok((leftover, _)) => {
                        Err(ErrorLocation {
                            file_name,
                            line: leftover.location_line(),
                            col:  leftover.get_column(),
                        })
                    }
                    Err(err) => {
                        eprintln!("ERROR: {}", err);
                        panic!("this should never happen - please report an issue at https://github.com/insou22/mipsy")            
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("ERROR: {}", err);
            panic!("this should never happen - please report an issue at https://github.com/insou22/mipsy")
        }
    }
}

const IDENT_FIRST_CHAR:  &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_";
const IDENT_CONTD_CHARS: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789.";

pub fn escape_char(char: char) -> String {
    match char {
        '\0' => "\\0".into(),
        '\r' => "\\r".into(),
        '\n' => "\\n".into(),
        '\t' => "\\t".into(),
        '\\' => "\\\'".into(),
        '\"' => "\\\"".into(),
        '\'' => "\\\'".into(),
        other => other.to_string(),
    }
}

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
    map(
        anychar,
        |char| char as u8,
    )(i)
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

pub fn tabs_to_spaces<T>(input: T) -> String
where
    T: AsRef<str>
{
    let mut string = String::new();
    
    let mut line_len = 0;
    for char in input.as_ref().chars() {
        if char == '\t' {
            let tab_size = 4 - (line_len % 4);
            line_len += tab_size;
            string.push_str(&" ".repeat(tab_size));
        } else if char == '\n' {
            line_len = 0;
            string.push(char);
        } else {
            line_len += 1;
            string.push(char);
        }
    }

    string
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
            ("ello".to_string(), b'h')
        );

        assert_eq!(
            unspan(parse_any1(span("h")).unwrap()),
            ("".to_string(), b'h')
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