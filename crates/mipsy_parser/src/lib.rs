extern crate nom;

use misc::parse_result;
use nom::combinator::map;
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a [u8]>;

pub use constant::{MpConst, MpConstValue, MpConstValueLoc};
pub use directive::MpDirective;
pub use instruction::{MpArgument, MpInstruction};
pub use misc::{tabs_to_spaces, ErrorLocation};
pub use number::{MpImmediate, MpImmediateBinaryOp, MpNumber};
pub use parser::{MpItem, MpProgram, TaggedFile};
pub use register::{MpOffsetOperator, MpRegister, MpRegisterIdentifier};

pub use parser::parse_mips;

pub fn parse_instruction<T>(input: T, tab_size: u32) -> Result<MpInstruction, ErrorLocation>
where
    T: AsRef<str>,
{
    let string = misc::tabs_to_spaces(input, tab_size);

    parse_result(
        Span::new(string.as_bytes()),
        None,
        instruction::parse_instruction,
    )
}

pub fn parse_argument<T>(input: T, tab_size: u32) -> Result<MpArgument, ErrorLocation>
where
    T: AsRef<str>,
{
    let string = misc::tabs_to_spaces(input, tab_size);

    parse_result(
        Span::new(string.as_bytes()),
        None,
        map(instruction::parse_argument, |(arg, _, _)| arg),
    )
}

mod attribute;
mod constant;
mod directive;
mod instruction;
mod label;
mod misc;
mod number;
pub mod parser;
mod register;
