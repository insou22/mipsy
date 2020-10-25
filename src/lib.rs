use wasm_bindgen::prelude::*;

pub mod error;
pub mod inst;
pub mod yaml;
pub mod util;
pub mod compile;
pub mod decompile;
pub mod runtime;

use inst::instruction::InstSet;

#[wasm_bindgen]
pub fn get_iset() -> *const InstSet {
    let yaml = yaml::parse::get_instructions();

    let iset = inst::instruction::InstSet::new(&yaml).unwrap();

    return &*Box::leak(Box::new(iset));
}