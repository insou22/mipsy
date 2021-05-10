use std::rc::Rc;

use crate::{TEXT_BOT, error::{InternalError, MipsyInternalResult, ToMipsyResult, compiler}};
use crate::inst::instruction::SignatureRef;
use crate::{MPProgram, MipsyResult};
use crate::inst::instruction::InstSet;
use super::{Binary, data::Segment};
use mipsy_parser::{
    MPInstruction,
    MPItem,
    MPDirective,
};

pub fn find_instruction<'a>(iset: &'a InstSet, inst: &MPInstruction) -> MipsyInternalResult<SignatureRef<'a>> {
    if let Some(native) = iset.find_native(inst) {
        Ok(SignatureRef::Native(&native))
    } else if let Some(pseudo) = iset.find_pseudo(inst) {
        Ok(SignatureRef::Pseudo(&pseudo))
    } else {
        let mut matching_names: Vec<SignatureRef<'a>> = vec![];
        let mut close_names:    Vec<SignatureRef<'a>> = vec![];

        let all_instns = iset.native_set.iter()
            .map(|native| SignatureRef::Native(native))
            .chain(
                iset.pseudo_set.iter()
                    .map(|pseudo| SignatureRef::Pseudo(pseudo))
            );

        for real_inst in all_instns {
            if real_inst.name() == inst.name() {
                matching_names.push(real_inst);
            } else if strsim::levenshtein(real_inst.name(), inst.name()) == 1 {
                close_names.push(real_inst);
            }
        }
        
        if !matching_names.is_empty() {
            return Err(
                InternalError::Compiler(
                    compiler::Error::InstructionBadFormat {
                        inst_ast: inst.clone(),
                        correct_formats: matching_names.iter().map(SignatureRef::cloned).collect(),
                    }
                )
            );
        }
        
        if !close_names.is_empty() {
            return Err(
                InternalError::Compiler(
                    compiler::Error::InstructionSimName {
                        inst_ast: inst.clone(),
                        similar_instns: close_names.iter().map(SignatureRef::cloned).collect(),
                    }
                )
            );
        }

        return Err(
            InternalError::Compiler(
                compiler::Error::UnknownInstruction {
                    inst_ast: inst.clone(),
                }
            )
        );
    }
}

pub fn instruction_length(iset: &InstSet, inst: &MPInstruction) -> MipsyInternalResult<usize> {
    Ok(
        match find_instruction(iset, inst)? {
            SignatureRef::Native(_) => 1,
            SignatureRef::Pseudo(pseudo) => pseudo.expand.len(),
        }
    )
}

pub fn compile1(binary: &Binary, iset: &InstSet, inst: &MPInstruction) -> MipsyInternalResult<Vec<u32>> {
    Ok(
        find_instruction(iset, inst)?.compile_ops(binary, iset, inst)?
    )
}

pub fn populate_text(binary: &mut Binary, iset: &InstSet, program: &MPProgram) -> MipsyResult<()> {
    let mut segment = Segment::Text;

    for (item, file_tag, line) in program.items().iter() {
        let file_tag = file_tag.clone().unwrap_or_else(|| Rc::from(""));
        let line = *line;

        match item {
            MPItem::Directive(directive) => match directive {
                MPDirective::Text  => segment = Segment::Text,
                MPDirective::Data  => segment = Segment::Data,
                MPDirective::KText => segment = Segment::KText,
                MPDirective::KData => segment = Segment::KData,
                _ => {}
            }
            MPItem::Instruction(ref instruction) => {
                let mut compiled = compile1(binary, iset, instruction)
                    .to_compiler_mipsy_result(file_tag.clone(), line, instruction.col(), instruction.col_end())?;

                let text = match segment {
                    Segment::Text  => {
                        if !file_tag.is_empty() {
                            binary.line_numbers.insert(TEXT_BOT + (binary.text.len() as u32) * 4, (file_tag.clone(), line));
                        }

                        &mut binary.text
                    }
                    Segment::KText => &mut binary.ktext,
                    _              => continue,
                };

                text.append(&mut compiled);
            }
            MPItem::Label(_) => {}
        }
    }

    Ok(())
}
