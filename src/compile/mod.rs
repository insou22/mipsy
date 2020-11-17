use crate::{error::CompileError, InstSet, MPProgram, MipsyResult, util::{Safe, cerr}};
use case_insensitive_hashmap::CaseInsensitiveHashMap;

mod bytes;

mod checker;
use checker::{
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
pub const DATA_BOT:  u32 = 0x10000000;
pub const HEAP_BOT:  u32 = 0x10008000;
pub const STACK_TOP: u32 = 0x7FFFFF00;
pub const KTEXT_BOT: u32 = 0x80000000;
pub const KDATA_BOT: u32 = 0x90000000;

pub struct Binary {
    pub text:    Vec<u32>,
    pub data:    Vec<Safe<u8>>,
    pub ktext:   Vec<u32>,
    pub kdata:   Vec<Safe<u8>>,
    pub labels:  CaseInsensitiveHashMap<u32>,
    pub breakpoints: Vec<u32>,
    pub globals: Vec<String>,
}

impl Binary {
    pub fn get_label(&self, label: &str) -> MipsyResult<u32> {
        if let Some(&addr) = self.labels.get(label) {
            Ok(addr)
        } else {
            let label_lower = label.to_ascii_lowercase();

            let similar = self.labels.keys()
                    .map(|label| label.to_ascii_lowercase())
                    .filter(|label| strsim::levenshtein(label, &label_lower) <= 3)
                    .collect();
            
            cerr(CompileError::UnresolvedLabel(label.to_string(), similar))
        }
    }

    pub fn insert_label(&mut self, label: &str, addr: u32) {
        self.labels.insert(label, addr);
    }
}

pub fn compile(program: &MPProgram, iset: &InstSet) -> MipsyResult<Binary> {
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

fn get_kernel() -> MPProgram {
    mipsy_parser::parse_mips(KERN_FILE).unwrap()
}
