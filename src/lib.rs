pub mod error;
pub mod inst;
pub mod yaml;
pub mod util;
pub mod compile;
pub mod decompile;
pub mod runtime;

pub use error::{
    RSpimResult,
    RSpimError,
};
pub use inst::instruction::InstSet;
pub use compile::{
    Binary,
    TEXT_BOT,
    DATA_BOT,
    HEAP_BOT,
    STACK_TOP,
    KTEXT_BOT
};
pub use rspim_parser::MPProgram;