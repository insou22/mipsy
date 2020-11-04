use crate::{
    RSpimResult,
    MPProgram,
};
use super::Context;
use rspim_parser::{
    MPInstruction,
};

pub fn instruction_length(inst: &MPInstruction) -> RSpimResult<usize> {
    Ok(1)
}

pub fn populate_text(context: &mut Context, program: &MPProgram) -> RSpimResult<()> {
    Ok(())
}
