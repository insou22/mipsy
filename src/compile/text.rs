use crate::{
    RSpimResult,
    MPProgram,
};
use crate::inst::instruction::InstSet;
use super::Binary;
use rspim_parser::{
    MPInstruction,
    MPItem,
    MPDirective,
};

pub fn instruction_length(iset: &InstSet, inst: &MPInstruction) -> RSpimResult<usize> {
    if let Some(_) = iset.find_native(inst) {
        Ok(1)
    } else if let Some(pseudo) = iset.find_pseudo(inst) {
        Ok(pseudo.expand.len())
    } else {
        println!("Instruction: {} {:?}", inst.name(), inst.arguments());
        todo!() // TODO - error inst not found
    }
}

pub fn populate_text(binary: &mut Binary, iset: &InstSet, program: &MPProgram) -> RSpimResult<()> {
    let mut text = true;

    for &item in program.items().iter() {
        match item {
            MPItem::Directive(directive) => match directive {
                MPDirective::Text => text = true,
                MPDirective::Data => text = false,
                _ => {}
            }
            MPItem::Instruction(instruction) => {
                if !text {
                    continue;
                }

                if let Some(native) = iset.find_native(instruction) {
                    binary.text.push(native.compile(binary, instruction.arguments())?);
                } else if let Some(pseudo) = iset.find_pseudo(instruction) {
                    binary.text.append(&mut pseudo.compile(iset, binary, instruction.arguments())?);
                }
            }
            MPItem::Label(_) => {}
        }
    }

    Ok(())
}
