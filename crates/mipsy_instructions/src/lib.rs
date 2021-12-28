//! # mipsy_instructions
//! 
//! This crate, on it's surface, seems incredibly useless.
//! It only exports a single function, `inst_set`, which
//! only calls a single macro, the `instruction_set!` macro.
//! 
//! The `instruction_set!` macro is used to parse the MIPS
//! instruction set and generate a Rust struct that can be
//! used to decode the instructions.
//! 
//! This macro, however, is *rather slow* and usually won't
//! need to be recompiled often. On my Ryzen 7 5800X, it
//! takes ~25 seconds to run the macro, and this computation
//! doesn't seem possible to parallelize as rustc seems to
//! only allocate a single thread for a proc-macro computaion.
//! 
//! By putting this macro invocation in a separate crate,
//! incremental compilation allows us to only compile this crate
//! once, and then only recompile it if the instruction set
//! changes (although this would have to be a manual `clean`).

use mipsy_lib::InstSet;
use mipsy_codegen::instruction_set;

/// Generates you the instruction set.
/// 
/// This struct carries a *lot* of data (on the heap),
/// so it's recommended to only call this once, and
/// avoid cloning it if possible.
/// 
/// # Example
/// 
/// ```
/// use mipsy_instructions::inst_set;
/// 
/// let inst_set = inst_set();
/// assert!(inst_set.native_set().len() > 0);
/// assert!(inst_set.pseudo_set().len() > 0);
/// ```
pub fn inst_set() -> InstSet {
    instruction_set!("../../mips.yaml")
}
