use std::rc::Rc;

use mipsy_parser::{MPArgument, MPImmediate, MPItem, MPNumber, MPRegister};

use crate::{Binary, MPProgram, MipsyResult, error::ToMipsyResult, inst::instruction::ToRegister};

pub enum Warning {

}

pub fn check_pre(program: &MPProgram) -> MipsyResult<Vec<Warning>> {
    let warnings = vec![];

    for (item, file_tag, line) in program.items() {
        let file_tag = file_tag.clone().unwrap_or_else(|| Rc::from(""));
        let line = *line;

        match item {
            MPItem::Instruction(ref instruction) => {
                for (argument, col, col_end) in instruction.arguments() {
                    match argument {
                        MPArgument::Register(register) => {
                            let ident = match register {
                                MPRegister::Normal(id) => id,
                                MPRegister::Offset(_, id) => id,
                            };

                            ident.to_register().to_compiler_mipsy_result(file_tag.clone(), line, *col, *col_end)?;
                        }
                        MPArgument::Number(_) => {}
                    }
                }
            }
            MPItem::Label(_) => {}
            MPItem::Directive(_) => {}
        }
    }

    // TODO

    Ok(warnings)
}

pub fn check_post_data_label(program: &MPProgram, binary: &Binary) -> MipsyResult<Vec<Warning>> {
    let warnings = vec![];

    for (item, file_tag, line) in program.items() {
        let file_tag = file_tag.clone().unwrap_or_else(|| Rc::from(""));
        let line = *line;

        match item {
            MPItem::Instruction(ref instruction) => {
                for (argument, col, col_end) in instruction.arguments() {
                    match argument {
                        MPArgument::Register(_) => {}
                        MPArgument::Number(number) => {
                            match number {
                                MPNumber::Immediate(imm) => {
                                    match imm {
                                        MPImmediate::LabelReference(label) => {
                                            binary.get_label(label)
                                                .to_compiler_mipsy_result(file_tag.clone(), line, *col, *col_end)?;
                                        }
                                        MPImmediate::I16(_) => {}
                                        MPImmediate::U16(_) => {}
                                        MPImmediate::I32(_) => {}
                                        MPImmediate::U32(_) => {}
                                    }
                                }
                                MPNumber::Float32(_) => {}
                                MPNumber::Float64(_) => {}
                                MPNumber::Char(_) => {}
                            }
                        }
                    }
                }
            }
            MPItem::Label(_) => {}
            MPItem::Directive(_) => {}
        }
    }

    // TODO

    Ok(warnings)
}
