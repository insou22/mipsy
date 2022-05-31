use std::{collections::HashMap, rc::Rc};
use crate::{InstSet, MpProgram, MipsyResult, error::{InternalError, MipsyInternalResult, compiler}, util::Safe};

mod bytes;

mod checker;
pub use checker::{
    check_pre,
    check_post_data_label,
};

mod data;
use data::populate_labels_and_data;

mod text;
use linked_hash_map::LinkedHashMap;
use mipsy_parser::TaggedFile;
use mipsy_utils::MipsyConfig;
use text::populate_text;

mod extra;


pub use text::compile1;

use self::extra::move_labels;


static KERN_FILE: &str = include_str!("../../../../kern.s");

pub const TEXT_BOT:   u32 = 0x00400000;
pub const TEXT_TOP:   u32 = 0x0FFFFFFF;
pub const GLOBAL_BOT: u32 = 0x10000000;
pub const GLOBAL_PTR: u32 = 0x10008000;
pub const DATA_BOT:   u32 = 0x10010000;
pub const HEAP_BOT:   u32 = 0x10040000;
pub const STACK_BOT:  u32 = 0x7FFF0000;
pub const STACK_PTR:  u32 = 0x7FFFFFFC;
pub const STACK_TOP:  u32 = 0x7FFFFFFF;
pub const KTEXT_BOT:  u32 = 0x80000000;
pub const KDATA_BOT:  u32 = 0x90000000;

pub struct Binary {
    pub text:    Vec<Safe<u8>>,
    pub data:    Vec<Safe<u8>>,
    pub ktext:   Vec<Safe<u8>>,
    pub kdata:   Vec<Safe<u8>>,
    pub labels:  LinkedHashMap<String, u32>,
    pub constants: HashMap<String, i64>,
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
                    .map(|label| (strsim::jaro_winkler(&label, &label_lower), label))
                    .filter(|&(sim, _)| sim >= 0.9)
                    .collect::<Vec<_>>();

            similar.sort_by(|&(sim1, _), (sim2, _)| sim1.partial_cmp(sim2).unwrap());

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
        self.labels.insert(label.to_string(), addr);
    }

    pub fn text_words(&'_ self) -> impl Iterator<Item = Safe<u32>> + '_ {
        (&self.text).chunks_exact(4)
            .into_iter()
            .map(|chunk| match (chunk[0], chunk[1], chunk[2], chunk[3]) {
                (Safe::Valid(b1), Safe::Valid(b2), Safe::Valid(b3), Safe::Valid(b4)) => {
                    Safe::Valid(u32::from_le_bytes([b1, b2, b3, b4]))
                }
                _ => Safe::Uninitialised,
            })

    }
}

#[derive(Debug, Default)]
pub struct CompilerOptions {
    moves: Vec<(String, String)>,
}

impl CompilerOptions {
    pub fn new(moves: Vec<(String, String)>) -> Self {
        Self {
            moves,
        }
    }

    pub fn moves(&self) -> impl Iterator<Item = (&str, &str)> {
        self.moves.iter()
            .map(|(old, new)| (old.as_str(), new.as_str()))
    }
}

pub fn compile(program: &mut MpProgram, config: &MipsyConfig, options: &CompilerOptions, iset: &InstSet) -> MipsyResult<Binary> {
    compile_with_kernel(program, &mut get_kernel(), options, config, iset)
}

pub fn compile_with_kernel(program: &mut MpProgram, kernel: &mut MpProgram, options: &CompilerOptions, config: &MipsyConfig, iset: &InstSet) -> MipsyResult<Binary> {
    let warnings = check_pre(program)?;
    if !warnings.is_empty() {
        // TODO: Deal with warnings here
    }

    let mut binary = Binary {
        text: vec![],
        data: vec![],
        ktext: vec![],
        kdata: vec![],
        labels: LinkedHashMap::new(),
        constants: HashMap::new(),
        breakpoints: vec![],
        globals: vec![],
        line_numbers: HashMap::new(),
    };
    
    populate_labels_and_data(&mut binary, config, iset, kernel)?;

    populate_labels_and_data(&mut binary, config, iset, program)?;

    let warnings = check_post_data_label(program, &binary)?;
    if !warnings.is_empty() {
        // TODO: Deal with warnings here
    }

    move_labels(&mut binary, options.moves());

    populate_text           (&mut binary, iset, config, program)?;

    populate_text           (&mut binary, iset, config, &kernel)?;

    Ok(binary)
}

pub fn get_kernel() -> MpProgram {
    // kernel file has tabsize of 8
    mipsy_parser::parse_mips(vec![TaggedFile::new(None, KERN_FILE)], 8)
        .expect("Kernel file should always build")
}
