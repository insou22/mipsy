use std::rc::Rc;

use crate::{Safe, TEXT_BOT, error::{InternalError, MipsyInternalResult, ToMipsyResult, compiler}};
use crate::inst::instruction::SignatureRef;
use crate::{MpProgram, MipsyResult};
use crate::inst::instruction::InstSet;
use super::{Binary, bytes::ToBytes, data::Segment};
use mipsy_parser::{MpInstruction, MpItem};
use mipsy_utils::MipsyConfig;

pub fn find_instruction<'a>(iset: &'a InstSet, inst: &MpInstruction) -> MipsyInternalResult<SignatureRef<'a>> {
    if let Some(native) = iset.find_native(inst) {
        Ok(SignatureRef::Native(native))
    } else if let Some(pseudo) = iset.find_pseudo(inst) {
        Ok(SignatureRef::Pseudo(pseudo))
    } else {
        let mut matching_names: Vec<SignatureRef<'a>> = vec![];
        let mut close_names:    Vec<SignatureRef<'a>> = vec![];

        let all_instns = iset.native_set().iter()
            .map(|native| SignatureRef::Native(native))
            .chain(
                iset.pseudo_set().iter()
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

        Err(
            InternalError::Compiler(
                compiler::Error::UnknownInstruction {
                    inst_ast: inst.clone(),
                }
            )
        )
    }
}

pub fn instruction_length(iset: &InstSet, inst: &MpInstruction) -> MipsyInternalResult<usize> {
    Ok(
        match find_instruction(iset, inst)? {
            SignatureRef::Native(_) => 1,
            SignatureRef::Pseudo(pseudo) => pseudo.expansion().len(),
        }
    )
}

pub fn compile1(binary: &Binary, iset: &InstSet, inst: &MpInstruction) -> MipsyInternalResult<Vec<u32>> {
    find_instruction(iset, inst)?.compile_ops(binary, iset, inst)
}

pub fn populate_text(binary: &mut Binary, iset: &InstSet, config: &MipsyConfig, program: &MpProgram) -> MipsyResult<()> {
    let mut segment = Segment::Text;

    for attributed_item in program.items() {
        let line = attributed_item.line_number();
        let file_tag = attributed_item.file_tag()
            .unwrap_or_else(|| Rc::from(""));
        let item = attributed_item.item();

        match item {
            MpItem::Directive(directive) => {
                let bytes = super::data::eval_directive(&directive.0, binary, config, file_tag.clone(), &mut segment, false)?;
                match segment {
                    Segment::Text  => {
                        binary.text.extend(bytes);
                    }
                    Segment::KText => {
                        binary.ktext.extend(bytes);
                    }
                    // already dealt with
                    Segment::Data | Segment::KData => {}
                }
            }
            MpItem::Instruction(ref instruction) => {
                let compiled = compile1(binary, iset, instruction)
                    .into_compiler_mipsy_result(file_tag.clone(), line, instruction.col(), instruction.col_end())?;

                let text = match segment {
                    Segment::Text  => {
                        let alignment = (4 - binary.text.len() % 4) % 4;
                        binary.text.append(&mut vec![Safe::Uninitialised; alignment]);

                        if !file_tag.is_empty() {
                            binary.line_numbers.insert(TEXT_BOT + (binary.text.len() as u32), (file_tag.clone(), line));
                        }

                        &mut binary.text
                    }
                    Segment::KText => {
                        let alignment = (4 - binary.ktext.len() % 4) % 4;
                        binary.ktext.append(&mut vec![Safe::Uninitialised; alignment]);
                        
                        &mut binary.ktext
                    },
                    _              => continue,
                };

                text.append(&mut compiled.into_iter().flat_map(|ref b| ToBytes::to_bytes(b)).map(Safe::Valid).collect());
            }
            MpItem::Label(_) => {}
            MpItem::Constant(_) => {}
        }
    }

    Ok(())
}
