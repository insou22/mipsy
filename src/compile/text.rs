use crate::compile::CompileError;
use crate::compile::cerr;
use crate::inst::instruction::SignatureRef;
use crate::{MPProgram, MipsyResult, util::WithLoc};
use crate::inst::instruction::InstSet;
use super::{Binary, data::Segment};
use mipsy_parser::{
    MPInstruction,
    MPItem,
    MPDirective,
};

pub fn find_instruction<'a>(iset: &'a InstSet, inst: &MPInstruction) -> MipsyResult<SignatureRef<'a>> {
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
            return cerr(
                CompileError::InstructionBadFormat(
                    inst.clone(),
                    matching_names.iter().map(SignatureRef::cloned).collect()
                )
            );
        }
        
        if !close_names.is_empty() {
            return cerr(
                CompileError::InstructionSimName(
                    inst.clone(),
                    close_names.iter().map(SignatureRef::cloned).collect()
                )
            );
        }

        return cerr(
            CompileError::UnknownInstruction(
                inst.clone()
            )
        );
    }
}

pub fn instruction_length(iset: &InstSet, inst: &MPInstruction) -> MipsyResult<usize> {
    Ok(
        match find_instruction(iset, inst)? {
            SignatureRef::Native(_) => 1,
            SignatureRef::Pseudo(pseudo) => pseudo.expand.len(),
        }
    )
}

pub fn compile1(binary: &Binary, iset: &InstSet, inst: &MPInstruction) -> MipsyResult<Vec<u32>> {
    Ok(
        find_instruction(iset, inst)?.compile_ops(binary, iset, inst)?
    )
}

pub fn populate_text(binary: &mut Binary, iset: &InstSet, program: &MPProgram) -> MipsyResult<()> {
    let mut segment = Segment::Text;

    for item in program.items().iter() {
        let line = item.1;

        match &item.0 {
            MPItem::Directive(directive) => match directive {
                MPDirective::Text  => segment = Segment::Text,
                MPDirective::Data  => segment = Segment::Data,
                MPDirective::KText => segment = Segment::KText,
                MPDirective::KData => segment = Segment::KData,
                _ => {}
            }
            MPItem::Instruction(ref instruction) => {
                let mut compiled = compile1(binary, iset, instruction).with_line(line).with_col(instruction.col()).with_col_end(instruction.col_end())?;

                let text = match segment {
                    Segment::Text  => &mut binary.text,
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
