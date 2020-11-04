use crate::misc::parse_ident;
use nom::{
    IResult,
    character::complete::{
        space0,
        char,
    },
    sequence::tuple,
};

pub fn parse_label(i: &[u8]) -> IResult<&[u8], String> {
    let (
        remaining_data,
        (
            label,
            _,
            _
        )
    ) = tuple((
            parse_ident,
            space0,
            char(':'),
    ))(i)?;

    Ok((remaining_data, label))
}
