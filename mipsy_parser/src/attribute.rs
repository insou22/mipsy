use nom::{
    IResult,
    bytes::complete::{
        tag,
        take_until
    },
    character::complete::multispace0,
    combinator::{
        map,
        opt
    },
    sequence::tuple
};

use crate::{
    Span,
    misc::parse_ident
};

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    key: String,
    value: Option<String>,
}

impl Attribute {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> Option<&str> {
        self.value.as_deref()
    }
}

fn parse_attribute<'a>(attribute_header: &'static str) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, Attribute> {
    move |i| {
        map(
            tuple((
                multispace0,
                tag(attribute_header),
                multispace0,
                parse_ident,
                multispace0,
                opt(
                    tuple((
                        tag("("),
                        multispace0,
                        take_until(")"),
                        multispace0,
                        tag(")"),
                    )),
                ),
                multispace0,
                tag("]"),
            )),
            |(
                _,
                _,
                _,
                key,
                _,
                value,
                _,
                _,
            )| {
                if let Some((_, _, value, _, _)) = value {
                    let text = String::from_utf8_lossy(
                        &value.iter()
                            .copied()
                            .collect::<Vec<_>>()
                    ).to_string();
    
                    Attribute {
                        key,
                        value: Some(text),
                    }
                } else {
                    Attribute {
                        key,
                        value: None,
                    }
                }
            }
        )(i)
    }
}

pub fn parse_outer_attribute(i: Span<'_>) -> IResult<Span<'_>, Attribute> {
    parse_attribute("#![")(i)
}

pub fn parse_inner_attribute(i: Span<'_>) -> IResult<Span<'_>, Attribute> {
    parse_attribute("#[")(i)   
}
