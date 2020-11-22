extern crate nom;

use nom::combinator::map;
use nom_locate::LocatedSpan;
use misc::parse_result;

pub type Span<'a> = LocatedSpan<&'a [u8]>;

pub use parser::MPProgram;
pub use parser::MPItem;
pub use instruction::{
    MPInstruction,
    MPArgument,
};
pub use directive::MPDirective;
pub use misc::{
    ErrorLocation,
    tabs_to_spaces,
};
pub use number::{
    MPNumber,
    MPImmediate,
};
pub use register::{
    MPRegister,
    MPRegisterIdentifier,
};


pub use parser::parse_mips;

pub fn parse_instruction<T>(input: T) -> Result<MPInstruction, ErrorLocation>
where
    T: AsRef<str>,
{
    let string = misc::tabs_to_spaces(input);

    parse_result(Span::new(string.as_bytes()), instruction::parse_instruction)
}

pub fn parse_argument<T>(input: T) -> Result<MPArgument, ErrorLocation>
where
    T: AsRef<str>,
{
    let string = misc::tabs_to_spaces(input);

    parse_result(
        Span::new(string.as_bytes()),
        map(
            instruction::parse_argument,
            |(arg, _, _)| arg
        )
    )
}

pub const VERSION: &str = concat!(env!("VERGEN_COMMIT_DATE"), " ", env!("VERGEN_SHA_SHORT"));

pub mod parser;
mod directive;
mod instruction;
mod label;
mod misc;
mod number;
mod register;