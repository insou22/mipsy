extern crate nom;


pub use parser::Program;
pub use parser::Item;
pub use instruction::{
    Instruction,
    Argument,
};
pub use directive::Directive;
pub use number::{
    Number,
    Immediate,
};
pub use register::{
    Register,
    RegisterIdentifier,
};


pub use parser::parse_mips;


pub mod parser;
mod directive;
mod instruction;
mod label;
mod misc;
mod number;
mod register;