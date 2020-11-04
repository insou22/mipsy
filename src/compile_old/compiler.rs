use crate::error::RSpimResult;
use super::context::Context;
use super::context::Binary;
use super::data_label_compiler;
use super::text_compiler;
use crate::inst::instruction::InstSet;
use crate::error::RSpimError;

use rspim_parser::Program;

pub const TEXT_BOT:  u32 = 0x00400000;
pub const DATA_BOT:  u32 = 0x10000000;
pub const HEAP_BOT:  u32 = 0x10008000;
pub const STACK_TOP: u32 = 0x7FFFFF00;
pub const KTEXT_BOT: u32 = 0x80000000;

pub const LITTLE_ENDIAN: bool = true;

pub fn generate(tokens: Program, iset: &InstSet) -> RSpimResult<Binary> {
    let mut context = Context::new(tokens);

    match generate_without_err_context(&mut context, iset) {
        Ok(_) => Ok(context.program),
        Err(RSpimError::Compile(err)) => Err(RSpimError::CompileContext(err, context)),
        Err(err) => Err(err),
    }
}

pub fn generate_without_err_context(context: &mut Context, iset: &InstSet) -> RSpimResult<()> {
    data_label_compiler::generate_labels_and_data(context, iset)?;
    context.reset_state();

    text_compiler::generate_text(context, iset)?;

    Ok(())
}
