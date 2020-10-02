use super::instruction::PseudoSignature;
use super::instruction::PseudoExpansion;
use super::instruction::InstFormat;
use super::instruction::InstSet;
use crate::error::RSpimResult;
use crate::cerr;
use crate::error::CompileError;
use crate::util::TruncImm;

pub trait PseudoInst {
    fn expand(&self, set: &InstSet, input: Vec<u32>) -> RSpimResult<Vec<u32>>;
}

struct Li;
impl PseudoInst for Li {
    fn expand(&self, set: &InstSet, input: Vec<u32>) -> RSpimResult<Vec<u32>> {
        if input.len() != 2 {
            return cerr!(CompileError::Unknown); // TODO
        }

        let rt = input[0];
        let imm = input[1];

        Ok(
            if imm as u32 & 0xFFFF0000 == imm as u32 & 0x8000 {
                vec![
                    set.find_instruction_exact("addiu", InstFormat::RtRsIm)?
                        .gen_op(&vec![0, rt, imm.trunc_imm()])?
                ]
            } else if imm & 0xFFFF == 0 {
                vec![
                    set.find_instruction_exact("lui", InstFormat::RtIm)?
                        .gen_op(&vec![rt, imm >> 16])?
                ]
            } else {
                vec![
                    set.find_instruction_exact("lui", InstFormat::RtIm)?
                        .gen_op(&vec![rt, imm >> 16])?,

                    set.find_instruction_exact("addiu", InstFormat::RtRsIm)?
                        .gen_op(&vec![0, rt, imm.trunc_imm()])?
                ]
            }
        )
    }
}

struct La;
impl PseudoInst for La {
    fn expand(&self, set: &InstSet, input: Vec<u32>) -> RSpimResult<Vec<u32>> {
        if input.len() != 2 {
            return cerr!(CompileError::Unknown); // TODO
        }

        let rt = input[0];
        let imm = input[1];

        Ok(
            if imm as u32 & 0xFFFF0000 == imm as u32 & 0x8000 {
                vec![
                    set.find_instruction_exact("addiu", InstFormat::RtRsIm)?
                        .gen_op(&vec![0, rt, imm.trunc_imm()])?
                ]
            } else if imm & 0xFFFF == 0 {
                vec![
                    set.find_instruction_exact("lui", InstFormat::RtIm)?
                        .gen_op(&vec![rt, imm >> 16])?
                ]
            } else {
                vec![
                    set.find_instruction_exact("lui", InstFormat::RtIm)?
                        .gen_op(&vec![rt, imm >> 16])?,

                    set.find_instruction_exact("addiu", InstFormat::RtRsIm)?
                        .gen_op(&vec![0, rt, imm.trunc_imm()])?
                ]
            }
        )
    }
}

pub fn get_complex_pseudo(name: &str) -> RSpimResult<Box<dyn PseudoInst>> {
    match name.to_ascii_lowercase().as_ref() {
        "li" => Ok(Box::new(Li)),
        "la" => Ok(Box::new(La)),
        _ => cerr!(CompileError::Unknown),
    }
}

impl PseudoInst for PseudoSignature {
    fn expand(&self, set: &InstSet, input: Vec<u32>) -> RSpimResult<Vec<u32>> {
        let mut insts = vec![];

        if let PseudoExpansion::Simple(expands) = &self.expand {
            for expand in expands {
                if let Some(format) = expand.format {
                    let inst = set.find_instruction_exact(&expand.inst, format)?;

                    insts.push(inst.gen_op(&input)?);
                }
            }
        }

        Ok(insts)
    }
}