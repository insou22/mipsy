use std::{collections::HashMap, rc::Rc};
use crate::{InstSet, MpProgram, MipsyResult, error::{InternalError, MipsyInternalResult, compiler}, util::Safe};
use case_insensitive_hashmap::CaseInsensitiveHashMap;

mod bytes;

mod checker;
pub use checker::{
    check_pre,
    check_post_data_label,
};

mod data;
use data::populate_labels_and_data;

mod text;
use text::populate_text;
pub use text::compile1;

static KERN_FILE: &str = include_str!("../../kern.s");

pub const TEXT_BOT:  u32 = 0x00400000;
pub const DATA_BOT:  u32 = 0x10010000;
pub const HEAP_BOT:  u32 = 0x10040000;
pub const STACK_TOP: u32 = 0x7FFFFF00;
pub const KTEXT_BOT: u32 = 0x80000000;
pub const KDATA_BOT: u32 = 0x90000000;

pub struct Binary {
    pub text:    Vec<u32>,
    pub data:    Vec<Safe<u8>>,
    pub ktext:   Vec<u32>,
    pub kdata:   Vec<Safe<u8>>,
    pub labels:  CaseInsensitiveHashMap<u32>,
    pub breakpoints:  Vec<u32>,
    pub globals: Vec<String>,
    pub line_numbers: HashMap<u32, (Rc<str>, u32)>,
}

impl Binary {
    pub fn get_label(&self, label: &str) -> MipsyInternalResult<u32> {
        if let Some(&addr) = self.labels.get(label) {
            Ok(addr)
        } else {
            let label_lower = label.to_ascii_lowercase();

            let mut similar = self.labels.keys()
                    .map(|label| label.to_ascii_lowercase())
                    .map(|label| (strsim::levenshtein(&label, &label_lower), label))
                    .filter(|&(sim, _)| sim <= 3)
                    .collect::<Vec<_>>();

            similar.sort_by_key(|&(sim, _)| sim);

            let similar = similar.into_iter()
                    .map(|(_, label)| label)
                    .collect::<Vec<_>>();

            Err(
                InternalError::Compiler(
                    compiler::Error::UnresolvedLabel {
                        label: label.to_string(),
                        similar,
                    }
                )
            )
        }
    }

    pub fn insert_label(&mut self, label: &str, addr: u32) {
        self.labels.insert(label, addr);
    }
}

pub fn compile(program: &MpProgram, iset: &InstSet) -> MipsyResult<Binary> {
    let warnings = check_pre(program)?;
    if !warnings.is_empty() {
        // TODO: Deal with warnings here
    }

    let mut binary = Binary {
        text: vec![],
        data: vec![],
        ktext: vec![],
        kdata: vec![],
        labels: CaseInsensitiveHashMap::new(),
        breakpoints: vec![],
        globals: vec![],
        line_numbers: HashMap::new(),
    };

    populate_labels_and_data(&mut binary, iset, &program)?;

    let warnings = check_post_data_label(program, &binary)?;
    if !warnings.is_empty() {
        // TODO: Deal with warnings here
    }

    populate_text           (&mut binary, iset, &program)?;
    
    let kernel = get_kernel();

    populate_labels_and_data(&mut binary, iset, &kernel)?;
    populate_text           (&mut binary, iset, &kernel)?;

    Ok(binary)
}

fn get_kernel() -> MpProgram {
    mipsy_parser::parse_mips(vec![(None, &KERN_FILE)]).unwrap()
}
