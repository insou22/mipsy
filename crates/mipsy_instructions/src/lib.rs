use mipsy_lib::InstSet;
use mipsy_codegen::instruction_set;

pub fn inst_set() -> InstSet {
    instruction_set!("../../mips.yaml")
}
