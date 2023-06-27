pub mod compile;
pub mod decompile;
pub mod error;
pub mod inst;
pub mod runtime;
pub mod util;

use std::rc::Rc;

use compile::CompilerOptions;
pub use mipsy_parser::MpProgram;

pub use compile::Binary;
pub use compile::{
    DATA_BOT, GLOBAL_BOT, GLOBAL_PTR, HEAP_BOT, KDATA_BOT, KTEXT_BOT, STACK_BOT, STACK_PTR,
    STACK_TOP, TEXT_BOT, TEXT_TOP,
};
pub use error::{
    runtime::Uninitialised, CompilerError, MipsyError, MipsyResult, ParserError, RuntimeError,
};
pub use inst::instruction::{ArgumentType, InstSet};
pub use inst::register::Register;
use mipsy_parser::TaggedFile;
use mipsy_utils::MipsyConfig;
pub use runtime::{Runtime, State};
pub use util::Safe;

pub fn compile(
    iset: &InstSet,
    files: Vec<TaggedFile<'_, '_>>,
    options: &CompilerOptions,
    config: &MipsyConfig,
) -> MipsyResult<Binary> {
    compile_with_kernel(iset, files, &mut compile::get_kernel(), options, config)
}

pub fn compile_with_kernel(
    iset: &InstSet,
    files: Vec<TaggedFile<'_, '_>>,
    kernel: &mut MpProgram,
    options: &CompilerOptions,
    config: &MipsyConfig,
) -> MipsyResult<Binary> {
    let mut parsed = mipsy_parser::parse_mips(files, config.tab_size).map_err(|err| {
        error::MipsyError::Parser(ParserError::new(
            error::parser::Error::ParseFailure,
            err.file_name.unwrap_or_else(|| Rc::from("")),
            err.line,
            err.col as u32,
        ))
    })?;

    let compiled = compile::compile_with_kernel(&mut parsed, kernel, options, config, iset)?;

    Ok(compiled)
}

pub use compile::compile1;

pub fn decompile(iset: &InstSet, binary: &Binary) -> String {
    decompile::decompile(binary, iset)
}

pub fn runtime(binary: &Binary, args: &[&str]) -> Runtime {
    runtime::Runtime::new(binary, args)
}
