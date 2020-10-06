use crate::error::RSpimResult;
use super::context::Token;
use super::context::Context;
use super::context::Program;
use super::data_label_compiler;
use super::text_compiler;
use crate::inst::instruction::InstSet;


pub const TEXT_BOT:  u32 = 0x00400000;
pub const DATA_BOT:  u32 = 0x10000000;
pub const STACK_TOP: u32 = 0x80000000;

pub const LITTLE_ENDIAN: bool = true;

pub fn generate(tokens: Vec<Token>, iset: &InstSet) -> RSpimResult<Program> {
    let mut context = Context::new(&tokens);

    data_label_compiler::generate_labels_and_data(&mut context, iset)?;
    context.reset_state();

    text_compiler::generate_text(&mut context, iset)?;

    Ok(context.program)
}
