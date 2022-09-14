use std::{collections::HashMap, rc::Rc};
use crate::{InstSet, MpProgram, MipsyResult, error::{InternalError, MipsyInternalResult, compiler}, util::Safe};
use serde::{Serialize, Deserialize};
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

// TODO(joshh): remove once if-let chaining is in
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Breakpoint {
    pub id: u32,
    pub enabled: bool,
    pub commands: Vec<String>,
}

impl Breakpoint {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            enabled: true,
            commands: Vec::new(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Binary {
    pub text:    Vec<Safe<u8>>,
    pub data:    Vec<Safe<u8>>,
    pub ktext:   Vec<Safe<u8>>,
    pub kdata:   Vec<Safe<u8>>,
    pub labels:  LinkedHashMap<String, u32>,
    pub constants: HashMap<String, i64>,
    pub globals: Vec<String>,
    pub line_numbers: HashMap<u32, (Rc<str>, u32)>,
    pub breakpoints: HashMap<u32, Breakpoint>,
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

    pub fn generate_breakpoint_id(&self) -> u32 {
        // TODO(joshh): reuses old breakpoint ids
        // this diverges from gdb behaviour but is it a problem?
        let mut id = self.breakpoints
                    .values()
                    .map(|bp| bp.id)
                    .fold(std::u32::MIN, |x, y| x.max(y))
                    .wrapping_add(1);

        if self.breakpoints.values().any(|bp| bp.id == id) {
            // find a free id to use
            // there's probably a neater way to do this,
            // but realistically if someone is using enough breakpoints
            // to fill a u32, they have bigger problems

            let mut ids = self.breakpoints
                    .values()
                    .map(|bp| bp.id)
                    .collect::<Vec<_>>();

            ids.sort_unstable();

            id = ids.into_iter()
                    .enumerate()
                    .find(|x| x.0 != x.1 as usize)
                    .expect("you've run out of breakpoints! why are you using so many")
                    .1;
        }

        id
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

    let mut binary = Binary::default();
    
    populate_labels_and_data(&mut binary, config, iset, kernel)?;

    populate_labels_and_data(&mut binary, config, iset, program)?;

    let warnings = check_post_data_label(program, &binary)?;
    if !warnings.is_empty() {
        // TODO: Deal with warnings here
    }

    move_labels(&mut binary, options.moves());

    populate_text(&mut binary, iset, config, program)?;

    populate_text(&mut binary, iset, config, kernel)?;

    Ok(binary)
}

pub fn get_kernel() -> MpProgram {
    // kernel file has tabsize of 8
    mipsy_parser::parse_mips(vec![TaggedFile::new(None, KERN_FILE)], 8)
        .expect("Kernel file should always build")
}
