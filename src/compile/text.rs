use crate::{MPProgram, MipsyResult};
use crate::inst::instruction::InstSet;
use super::{Binary, data::Segment};
use mipsy_parser::{
    MPInstruction,
    MPItem,
    MPDirective,
};

pub fn instruction_length(iset: &InstSet, inst: &MPInstruction) -> MipsyResult<usize> {
    if iset.find_native(inst).is_some() {
        Ok(1)
    } else if let Some(pseudo) = iset.find_pseudo(inst) {
        Ok(pseudo.expand.len())
    } else {
        println!("Instruction: {} {:?}", inst.name(), inst.arguments());
        todo!() // TODO - error inst not found
    }
}

pub fn compile1(binary: &Binary, iset: &InstSet, instruction: &MPInstruction) -> MipsyResult<Vec<u32>> {
    Ok(
        if let Some(native) = iset.find_native(instruction) {
            vec![native.compile(binary, instruction.arguments())?]
        } else if let Some(pseudo) = iset.find_pseudo(instruction) {
            pseudo.compile(iset, binary, instruction.arguments())?
        } else {
            todo!()
        }
    )
}

pub fn populate_text(binary: &mut Binary, iset: &InstSet, program: &MPProgram) -> MipsyResult<()> {
    let mut segment = Segment::Text;

    for item in program.items().iter() {
        match item {
            MPItem::Directive(directive) => match directive {
                MPDirective::Text  => segment = Segment::Text,
                MPDirective::Data  => segment = Segment::Data,
                MPDirective::KText => segment = Segment::KText,
                MPDirective::KData => segment = Segment::KData,
                _ => {}
            }
            MPItem::Instruction(ref instruction) => {
                let mut compiled = compile1(binary, iset, instruction)?;

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
