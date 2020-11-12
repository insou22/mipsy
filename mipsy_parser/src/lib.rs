extern crate nom;

use nom_locate::LocatedSpan;
pub type Span<'a> = LocatedSpan<&'a [u8]>;

pub use parser::MPProgram;
pub use parser::MPItem;
pub use instruction::{
    MPInstruction,
    MPArgument,
};
pub use directive::MPDirective;
pub use number::{
    MPNumber,
    MPImmediate,
};
pub use register::{
    MPRegister,
    MPRegisterIdentifier,
};


pub use parser::parse_mips;
pub fn parse_instruction<T>(input: T) -> Result<MPInstruction, &'static str>
where
    T: AsRef<str>,
{
    instruction::parse_instruction(Span::new(input.as_ref().as_bytes()))
        .map(|(_leftover, inst)| inst)
        .map_err(|_| "")
}

pub fn parse_argument<T>(input: T) -> Result<MPArgument, &'static str>
where
    T: AsRef<str>,
{
    instruction::parse_argument(Span::new(input.as_ref().as_bytes()))
        .map(|(_leftover, inst)| inst)
        .map_err(|_| "")
}

pub const VERSION: &str = concat!(env!("VERGEN_COMMIT_DATE"), " ", env!("VERGEN_SHA_SHORT"));

pub mod parser;
mod directive;
mod instruction;
mod label;
mod misc;
mod number;
mod register;