use crate::{error::CompileError, InstSet, MPProgram, MipsyResult, cerr, util::Safe};
use case_insensitive_hashmap::CaseInsensitiveHashMap;

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
    pub labels:  CaseInsensitiveHashMap<u32>,
    pub globals: Vec<String>,
}

impl Binary {
    pub fn get_label(&self, label: &str) -> MipsyResult<u32> {
        if let Some(&addr) = self.labels.get(label) {
            Ok(addr)
        } else {
            cerr!(CompileError::UnresolvedLabel(label.to_string()))
        }
    }

    pub fn insert_label(&mut self, label: &str, addr: u32) {
        self.labels.insert(label, addr);
    }
}

pub fn compile(program: &MPProgram, iset: &InstSet) -> MipsyResult<Binary> {
    let warnings = check_program(program)?;
    if !warnings.is_empty() {
        // TODO: Deal with warnings here
    }

    let mut binary = Binary {
        text: vec![],
        data: vec![],
        labels: CaseInsensitiveHashMap::new(),
        globals: vec![],
    };

    populate_labels_and_data(&mut binary, iset, program)?;
    populate_text           (&mut binary, iset, program)?;

    Ok(binary)
}
