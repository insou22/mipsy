extern crate nom;


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


pub mod parser;
mod directive;
mod instruction;
mod label;
mod misc;
mod number;
mod register;