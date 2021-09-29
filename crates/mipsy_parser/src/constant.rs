use nom::IResult;
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::tuple;
use nom::branch::alt;
use nom_locate::position;

use crate::Span;
use crate::misc::{comment_multispace0, parse_ident};
use crate::number::{parse_u32, parse_char};
use crate::parser::Position;

#[derive(Debug, Clone, PartialEq)]
pub struct MpConst {
    label: String,
    value: MpConstValueLoc,
    line: u32,
    col: u32,
    line_end: u32,
    col_end: u32,
}

impl MpConst {
    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn value(&self) -> &MpConstValueLoc {
        &self.value
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn col(&self) -> u32 {
        self.col
    }

    pub fn line_end(&self) -> u32 {
        self.line_end
    }

    pub fn col_end(&self) -> u32 {
        self.col_end
    }
}

pub type MpConstValueLoc = (MpConstValue, Position);

#[derive(Debug, Clone, PartialEq)]
pub enum MpConstValue {
    Value(u64),
    Const(String),
    Minus(Box<MpConstValueLoc>),
    Mult (Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Sum  (Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Sub  (Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Div  (Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Mod  (Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    And  (Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Or   (Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Xor  (Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Neg  (Box<MpConstValueLoc>),
    Shl  (Box<MpConstValueLoc>, Box<MpConstValueLoc>),
    Shr  (Box<MpConstValueLoc>, Box<MpConstValueLoc>),
}

pub fn parse_constant(i: Span<'_>) -> IResult<Span<'_>, MpConst> {
    map(
        tuple((
            position,
            parse_ident,
            comment_multispace0,
            char('='),
            comment_multispace0,
            parse_constant_value,
            position,
        )),
        |(pos_start, label, _, _, _, value, pos_end)| MpConst {
            label,
            value,
            line: pos_start.location_line(),
            col: pos_start.get_column() as _,
            line_end: pos_end.location_line(),
            col_end: pos_end.get_column() as _,
        }
    )(i)
}

pub fn parse_constant_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    parse_or_value(i)
}

pub fn parse_or_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    alt((
        map(
            tuple((
                position,
                parse_xor_value,
                many0(
                    map(
                        tuple((
                            comment_multispace0,
                            char('|'),
                            comment_multispace0,
                            parse_xor_value,
                        )),
                        |(_, _, _, value)| value,
                    )
                ),
                position,
            )),
            |(pos_start, value, values, pos_end)| {
                values.into_iter().fold(value, |acc, value| {
                    (MpConstValue::Or(Box::new(acc), Box::new(value)), Position::from_positions(pos_start, pos_end))
                })
            }
        ),
        parse_xor_value,
    ))(i)
}

pub fn parse_xor_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    alt((
        map(
            tuple((
                position,
                parse_and_value,
                many0(
                    map(
                        tuple((
                            comment_multispace0,
                            char('^'),
                            comment_multispace0,
                            parse_and_value,
                        )),
                        |(_, _, _, value)| value,
                    )
                ),
                position,
            )),
            |(pos_start, value, values, pos_end)| {
                values.into_iter().fold(value, |acc, value| {
                    (MpConstValue::Xor(Box::new(acc), Box::new(value)), Position::from_positions(pos_start, pos_end))
                })
            }
        ),
        parse_and_value,
    ))(i)
}

pub fn parse_and_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    alt((
        map(
            tuple((
                position,
                parse_shift_value,
                many0(
                    map(
                        tuple((
                            comment_multispace0,
                            char('&'),
                            comment_multispace0,
                            parse_shift_value,
                        )),
                        |(_, _, _, value)| value,
                    )
                ),
                position,
            )),
            |(pos_start, value, values, pos_end)| {
                values.into_iter().fold(value, |acc, value| {
                    (MpConstValue::And(Box::new(acc), Box::new(value)), Position::from_positions(pos_start, pos_end))
                })
            }
        ),
        parse_shift_value,
    ))(i)
}

pub fn parse_shift_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    map(
        tuple((
            position,
            parse_add_sub_value,
            many0(
                map(
                    tuple((
                        comment_multispace0,
                        alt((
                            tag("<<"),
                            tag(">>"),
                        )),
                        comment_multispace0,
                        parse_add_sub_value,
                    )),
                    |(_, tag, _, value)| (tag, value),
                )
            ),
            position,
        )),
        |(pos_start, value, values, pos_end)| {
            values.into_iter().fold(value, |acc, (tag, value)| {
                match *tag.fragment() {
                    b"<<" => (MpConstValue::Shl(Box::new(acc), Box::new(value)), Position::from_positions(pos_start, pos_end)),
                    b">>" => (MpConstValue::Shr(Box::new(acc), Box::new(value)), Position::from_positions(pos_start, pos_end)),
                    _     => unreachable!(),
                }
            })
        }
    )(i)
}

pub fn parse_add_sub_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    map(
        tuple((
            position,
            parse_mul_div_mod_value,
            many0(
                map(
                    tuple((
                        comment_multispace0,
                        alt((
                            char('+'),
                            char('-'),
                        )),
                        comment_multispace0,
                        parse_mul_div_mod_value,
                    )),
                    |(_, tag, _, value)| (tag, value),
                )
            ),
            position,
        )),
        |(pos_start, value, values, pos_end)| {
            values.into_iter().fold(value, |acc, (tag, value)| {
                match tag {
                    '+' => (MpConstValue::Sum(Box::new(acc), Box::new(value)), Position::from_positions(pos_start, pos_end)),
                    '-' => (MpConstValue::Sub(Box::new(acc), Box::new(value)), Position::from_positions(pos_start, pos_end)),
                    _   => unreachable!(),
                }
            })
        }
    )(i)
}

pub fn parse_mul_div_mod_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    map(
        tuple((
            position,
            parse_unary_op_value,
            many0(
                map(
                    tuple((
                        comment_multispace0,
                        alt((
                            char('*'),
                            char('/'),
                            char('%'),
                        )),
                        comment_multispace0,
                        parse_unary_op_value,
                    )),
                    |(_, tag, _, value)| (tag, value),
                )
            ),
            position,
        )),
        |(pos_start, value, values, pos_end)| {
            values.into_iter().fold(value, |acc, (tag, value)| {
                match tag {
                    '*' => (MpConstValue::Mult(Box::new(acc), Box::new(value)), Position::from_positions(pos_start, pos_end)),
                    '/' => (MpConstValue::Div (Box::new(acc), Box::new(value)), Position::from_positions(pos_start, pos_end)),
                    '%' => (MpConstValue::Mod (Box::new(acc), Box::new(value)), Position::from_positions(pos_start, pos_end)),
                    _   => unreachable!(),
                }
            })
        }
    )(i)
}

pub fn parse_unary_op_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    alt((
        parse_plus_value,
        parse_minus_value,
        parse_neg_value,
        parse_value,
    ))(i)
}

pub fn parse_plus_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    map(
        tuple((
            char('+'),
            comment_multispace0,
            parse_unary_op_value,
        )),
        |(_, _, value)| value
    )(i)
}

pub fn parse_minus_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    map(
        tuple((
            position,
            char('-'),
            comment_multispace0,
            parse_unary_op_value,
            position,
        )),
        |(pos_start, _, _, value, pos_end)| (MpConstValue::Minus(Box::new(value)), Position::from_positions(pos_start, pos_end)),
    )(i)
}

pub fn parse_neg_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    map(
        tuple((
            position,
            char('~'),
            comment_multispace0,
            parse_unary_op_value,
            position,
        )),
        |(pos_start, _, _, value, pos_end)| (MpConstValue::Neg(Box::new(value)), Position::from_positions(pos_start, pos_end))
    )(i)
}

pub fn parse_value(i: Span<'_>) -> IResult<Span<'_>, MpConstValueLoc> {
    alt((
        map(
            tuple((
                position,
                parse_u32,
                position,
            )),
            |(pos_start, value, pos_end)| (MpConstValue::Value(value as u64), Position::from_positions(pos_start, pos_end)),
        ),
        map(
            tuple((
                position,
                parse_char,
                position,
            )),
            |(pos_start, value, pos_end)| (MpConstValue::Value(value as u64), Position::from_positions(pos_start, pos_end)),
        ),
        map(
            tuple((
                position,
                parse_ident,
                position,
            )),
            |(pos_start, value, pos_end)| (MpConstValue::Const(value), Position::from_positions(pos_start, pos_end)),
        ),
        map(
            tuple((
                char('('),
                comment_multispace0,
                parse_constant_value,
                comment_multispace0,
                char(')'),
            )),
            |(_, _, value, _, _)| value,
        )
    ))(i)
}

#[cfg(test)]
mod test {
    use super::*;

    fn b<T>(t: T) -> Box<T> {
        Box::new(t)
    }

    #[test]
    fn test_parse_const_value() {
        use super::MpConstValue::*;

        // assert_eq!(
        //     parse_constant_value(Span::new("1".as_bytes())).unwrap().1,
        //     Value(1)
        // );

        // assert_eq!(
        //     parse_constant_value(Span::new("0x123".as_bytes())).unwrap().1,
        //     Value(0x123)
        // );

        // assert_eq!(
        //     parse_constant_value(Span::new("1 + 2 - 3 + 4 - 5".as_bytes())).unwrap().1,
        //     Sub(b(Sum(b(Sub(b(Sum(b(Value(1)), b(Value(2)))), b(Value(3)))), b(Value(4)))), b(Value(5)))
        // );

        // assert_eq!(
        //     parse_constant_value(Span::new("1 | 3 & 012 + 32 / 1 * 50 ^ ~5 - -3 % (3 >> 2 | abc + ++2 << 3)".as_bytes())).unwrap().1,
        //     Or(
        //         b(Value(1)), 
        //         b(Xor(
        //             b(And(
        //                 b(Value(3)), 
        //                 b(Sum(
        //                     b(Value(10)), 
        //                     b(Mult(
        //                         b(Div(
        //                             b(Value(32)),
        //                             b(Value(1)))),
        //                         b(Value(50))
        //                     ))
        //                 ))
        //             )),
        //             b(Sub(
        //                 b(Neg(
        //                     b(Value(5))
        //                 )),
        //                 b(Mod(
        //                     b(Minus(
        //                         b(Value(3))
        //                     )),
        //                     b(Or(
        //                         b(Shr(
        //                             b(Value(3)),
        //                             b(Value(2))
        //                         )),
        //                         b(Shl(
        //                             b(Sum(
        //                                 b(Const("abc".to_string())),
        //                                 b(Value(2))
        //                             )),
        //                             b(Value(3)))
        //                         ))
        //                     ))
        //                 ))
        //             ))
        //         ))
        // );
    }
}