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
        multispace0,
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

pub fn parse_escaped_char(i: &[u8]) -> IResult<&[u8], char> {
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

pub fn parse_ident(i: &[u8]) -> IResult<&[u8], String> {
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
        ident.push_str(std::str::from_utf8(rem).unwrap());
    }

    Ok((remaining_data, ident))
}

pub fn parse_any1(i: &[u8]) -> IResult<&[u8], u8> {
    if i.len() > 0 {
        Ok((&i[1..], i[0]))
    } else {
        Err(Error(ParseError::from_error_kind(i, ErrorKind::Eof)))
    }
}

pub fn comment_multispace0(i: &[u8]) -> IResult<&[u8], ()> {
    map(
        many0(
            tuple((
                multispace0,
                opt(
                    tuple((
                        preceded(char('#'), many0(none_of("\n"))),
                        char('\n'),
                    ))
                ),
            )),
        ),
        |_| ()
    )(i)
}

pub fn comment_multispace1(i: &[u8]) -> IResult<&[u8], ()> {
    map(
        many1(
            alt((
                map(
                    tuple((
                        multispace1,
                        opt(
                            tuple((
                                preceded(char('#'), many0(none_of("\n"))),
                                char('\n'),
                            ))
                        ),
                    )),
                    |_| (),
                ),
                map(
                    tuple((
                        preceded(char('#'), many0(none_of("\n"))),
                        char('\n'),
                    )),
                    |_| (),
                ),
                
            )),
        ),
        |_| ()
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_any1() {
        assert_eq!(
            parse_any1("hello".as_bytes()),
            Ok(("ello".as_bytes(), 'h' as u8))
        );

        assert_eq!(
            parse_any1("h".as_bytes()),
            Ok(("".as_bytes(), 'h' as u8))
        );
    }

    #[test]
    fn test_parse_ident() {
        assert_eq!(
            parse_ident("i".as_bytes()),
            Ok(("".as_bytes(), "i".into()))
        );

        assert_eq!(
            parse_ident("abc".as_bytes()),
            Ok(("".as_bytes(), "abc".into()))
        );

    }
}