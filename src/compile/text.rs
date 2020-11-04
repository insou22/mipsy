use crate::{
    RSpimResult,
    MPProgram,
};
use super::Context;
use crate::inst::instruction::InstSet;
use rspim_parser::{
    MPInstruction,
};

pub fn instruction_length(iset: &InstSet, inst: &MPInstruction) -> RSpimResult<usize> {
    if let Some(_) = iset.find_native(inst) {
        Ok(1)
    } else if let Some(pseudo) = iset.find_pseudo(inst) {
        Ok(pseudo.expand.len())
    } else {
        todo!() // TODO - error inst not found
    }
}

pub fn populate_text(context: &mut Context, iset: &InstSet, program: &MPProgram) -> RSpimResult<()> {
    // TODO
    Ok(())
}
