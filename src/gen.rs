use crate::context::*;
use crate::data_label_gen;
use crate::lexer::Token;

pub type GenRes<T> = Result<T, String>;

pub const TEXT_BOT: Address = 0x00400000;
pub const DATA_BOT: Address = 0x10000000;
pub const STACK_TOP: Address = 0x80000000;
// TODO: Kernel data
pub const LITTLE_ENDIAN: bool = true;

pub fn generate(tokens: Vec<Token>) -> GenRes<Program> {
    let mut context = Context::new(&tokens);

    data_label_gen::generate_labels_and_data(&mut context)?;
    context.reset_state();

    Ok(context.program)
}

fn ok<T: Default>() -> GenRes<T> {
    Ok(T::default())
}
