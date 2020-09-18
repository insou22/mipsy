use crate::instructions::*;
use crate::types::*;

pub fn instruction_length(name: &str) -> usize {
    let name = name.to_ascii_lowercase();

    for inst in &R_INSTRUCTIONS {
        if name.eq_ignore_ascii_case(inst.info().name) {
            return 1;
        }
    }

    for inst in &I_INSTRUCTIONS {
        if name.eq_ignore_ascii_case(inst.info().name) {
            return 1;
        }
    }

    for inst in &J_INSTRUCTIONS {
        if name.eq_ignore_ascii_case(inst.info().name) {
            return 1;
        }
    }

    for inst in &R_PSEUDO_INSTRUCTIONS {
        if name.eq_ignore_ascii_case(inst.name()) {
            
        }
    }

    1
}