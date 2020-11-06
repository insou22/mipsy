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
pub(crate) use rspim_parser::MPProgram;

pub use error::{
    RSpimResult,
    RSpimError,
};
pub use inst::instruction::InstSet;
pub use compile::Binary;
pub use runtime::Runtime;

pub fn inst_set() -> RSpimResult<InstSet> {
    let yaml = yaml::get_instructions();
    InstSet::new(&yaml)
}

pub fn compile(iset: &InstSet, program: &str) -> RSpimResult<Binary> {
    let parsed = rspim_parser::parse_mips(program)
            .map_err(|string| RSpimError::Compile(error::CompileError::Str(string)))?;
    let compiled = compile::compile(&parsed, &iset)?;

    Ok(compiled)
}

pub fn decompile(iset: &InstSet, binary: &Binary) -> String {
    decompile::decompile(binary, iset)
}

pub fn run(binary: &Binary) -> RSpimResult<Runtime> {
    runtime::Runtime::new(binary)
}
