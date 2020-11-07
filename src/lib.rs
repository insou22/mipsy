pub(crate) mod error;
pub(crate) mod inst;
pub(crate) mod yaml;
pub(crate) mod util;
pub(crate) mod compile;
pub        mod decompile;
pub(crate) mod runtime;

pub(crate) use compile::{
    TEXT_BOT,
    DATA_BOT,
    HEAP_BOT,
    STACK_TOP,
    KTEXT_BOT
};
pub(crate) use mipsy_parser::MPProgram;

pub use error::{
    MipsyResult,
    MipsyError,
};
pub use inst::instruction::InstSet;
pub use compile::Binary;
pub use runtime::{
    Runtime,
    RuntimeHandler,
    flags,
    mode,
    len,
    fd,
    n_bytes,
    void_ptr,
};

pub fn inst_set() -> MipsyResult<InstSet> {
    let yaml = yaml::get_instructions();
    InstSet::new(&yaml)
}

pub fn compile(iset: &InstSet, program: &str) -> MipsyResult<Binary> {
    let parsed = mipsy_parser::parse_mips(program)
            .map_err(|string| MipsyError::Compile(error::CompileError::Str(string)))?;
    let compiled = compile::compile(&parsed, &iset)?;

    Ok(compiled)
}

pub fn decompile(iset: &InstSet, binary: &Binary) -> String {
    decompile::decompile(binary, iset)
}

pub fn run(binary: &Binary) -> MipsyResult<Runtime> {
    runtime::Runtime::new(binary)
}
