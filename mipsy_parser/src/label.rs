use crate::{
    Span,
    misc::parse_ident,
};
use nom::{
    IResult,
    character::complete::{
        space0,
        char,
    },
    sequence::tuple,
};
use nom_locate::position;

#[derive(Debug, Clone, PartialEq)]
pub struct MpLabel {
    label:   String,
    col:     u32,
    col_end: u32,
}

impl MpLabel {
    pub fn label(&self) -> String {
        self.label.to_string()
    }

    pub fn col(&self) -> u32 {
        self.col
    }

    pub fn col_end(&self) -> u32 {
        self.col_end
    }
}

pub fn parse_label(i: Span<'_>) -> IResult<Span<'_>, MpLabel> {
    let (
        remaining_data,
        (
            pos_start,
            label,
            _,
            _,
            pos_end,
        )
    ) = tuple((
            position,
            parse_ident,
            space0,
            char(':'),
            position,
    ))(i)?;

    let col = pos_start.get_column() as u32;
    let col_end = pos_end.get_column() as u32;

    Ok((remaining_data, MpLabel { label, col, col_end }))
}
