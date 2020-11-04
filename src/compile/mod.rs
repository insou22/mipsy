use crate::{
    RSpimResult,
    MPProgram,
    InstSet,
    util::Safe,
};
use std::collections::HashMap;

mod bytes;

mod checker;
use checker::check_program;

mod data;
use data::populate_labels_and_data;

mod text;
use text::populate_text;


pub const TEXT_BOT:  u32 = 0x00400000;
pub const DATA_BOT:  u32 = 0x10000000;
pub const HEAP_BOT:  u32 = 0x10008000;
pub const STACK_TOP: u32 = 0x7FFFFF00;
pub const KTEXT_BOT: u32 = 0x80000000;

pub struct Binary {
    pub text:    Vec<u32>,
    pub data:    Vec<Safe<u8>>,
    pub labels:  HashMap<String, u32>,
    pub globals: Vec<String>,
}

struct Context {
    binary: Binary,
    segment: Segment,
    unmapped_text_len: usize,
}

#[derive(PartialEq)]
enum Segment {
    Text,
    Data,
}

pub fn compile(program: &MPProgram, iset: &InstSet) -> RSpimResult<Binary> {
    let warnings = check_program(program)?;
    if !warnings.is_empty() {
        // TODO: Deal with warnings here
    }

    let context = Context {
        binary: Binary {
            text: vec![],
            data: vec![],
            labels: HashMap::new(),
            globals: vec![],
        },
        segment: Segment::Text,
        unmapped_text_len: 0,
    };

    populate_labels_and_data(&mut context, program)?;
    populate_text           (&mut context, program)?;

    Ok(context.binary)
}
