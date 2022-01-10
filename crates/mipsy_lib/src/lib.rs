pub mod error;
pub mod inst;
pub mod util;
pub mod compile;
pub mod decompile;
pub mod runtime;

use std::rc::Rc;

pub use mipsy_parser::MpProgram;

pub use error::{
    MipsyResult,
    MipsyError,
    ParserError,
    CompilerError,
    RuntimeError,
    runtime::Uninitialised,
};
pub use inst::instruction::{
    InstSet,
    ArgumentType,
};
pub use inst::register::Register;
pub use compile::{Binary};
use mipsy_parser::TaggedFile;
use mipsy_utils::MipsyConfig;
pub use runtime::{
    Runtime,
    State,
};
pub use compile::{
    TEXT_BOT,
    GLOBAL_BOT,
    GLOBAL_PTR,
    DATA_BOT,
    HEAP_BOT,
    STACK_BOT,
    STACK_PTR,
    STACK_TOP,
    KTEXT_BOT,
    KDATA_BOT,
};
pub use util::Safe;

pub fn compile(iset: &InstSet, files: Vec<TaggedFile<'_, '_>>, config: &MipsyConfig) -> MipsyResult<Binary> {
    let mut parsed = mipsy_parser::parse_mips(files, config.tab_size)
        .map_err(|err| 
            error::MipsyError::Parser(
                ParserError::new(
                    error::parser::Error::ParseFailure,
                    err.file_name.unwrap_or_else(|| Rc::from("")),
                    err.line,
                    err.col as u32
                )
            )
        )?;

    let compiled = compile::compile(&mut parsed, config, iset)?;

    Ok(compiled)
}

pub use compile::compile1;

pub fn decompile(iset: &InstSet, binary: &Binary) -> String {
    decompile::decompile(binary, iset)
}

pub fn runtime(binary: &Binary, args: &[&str]) -> Runtime {
    runtime::Runtime::new(binary, args)
}

pub const VERSION: &str = concat!(env!("VERGEN_COMMIT_DATE"), " ", env!("VERGEN_SHA_SHORT"));
