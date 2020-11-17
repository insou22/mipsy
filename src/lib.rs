pub(crate) mod error;
pub(crate) mod inst;
pub(crate) mod yaml;
pub(crate) mod util;
pub(crate) mod compile;
pub        mod decompile;
pub(crate) mod runtime;
pub(crate) use mipsy_parser::MPProgram;

pub use error::{
    MipsyResult,
    MipsyError,
    CompileError,
    RuntimeError,
};
pub use inst::instruction::InstSet;
pub use inst::register::Register;
pub use compile::Binary;
pub use runtime::{
    Runtime,
    State,
    RuntimeHandler,
    flags,
    mode,
    len,
    fd,
    n_bytes,
    void_ptr,
};
pub use compile::{
    TEXT_BOT,
    DATA_BOT,
    HEAP_BOT,
    STACK_TOP,
    KTEXT_BOT,
    KDATA_BOT,
};

pub fn inst_set() -> MipsyResult<InstSet> {
    let yaml = yaml::get_instructions();
    InstSet::new(&yaml)
}

pub fn compile(iset: &InstSet, program: &str) -> MipsyResult<Binary> {
    let parsed = mipsy_parser::parse_mips(program)
            .map_err(|err| error::MipsyError::Compile(error::CompileError::ParseFailure { line: err.line, col: err.col }))?;
    let compiled = compile::compile(&parsed, &iset)?;

    Ok(compiled)
}

pub use compile::compile1;

pub fn decompile(iset: &InstSet, binary: &Binary) -> String {
    decompile::decompile(binary, iset)
}

pub fn run(binary: &Binary) -> MipsyResult<Runtime> {
    runtime::Runtime::new(binary)
}

pub const VERSION: &str = concat!(env!("VERGEN_COMMIT_DATE"), " ", env!("VERGEN_SHA_SHORT"));
