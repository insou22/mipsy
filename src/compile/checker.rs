use std::rc::Rc;

use mipsy_parser::{MpArgument, MpImmediate, MpItem, MpNumber, MpRegister};

use crate::{Binary, MpProgram, MipsyResult, error::ToMipsyResult, inst::instruction::ToRegister};

pub enum Warning {

}

pub fn check_pre(program: &MpProgram) -> MipsyResult<Vec<Warning>> {
    let warnings = vec![];

    for (item, file_tag, line) in program.items() {
        let file_tag = file_tag.clone().unwrap_or_else(|| Rc::from(""));
        let line = *line;

        match item {
            MpItem::Instruction(ref instruction) => {
                for (argument, col, col_end) in instruction.arguments() {
                    match argument {
                        MpArgument::Register(register) => {
                            let ident = match register {
                                MpRegister::Normal(id) => id,
                                MpRegister::Offset(_, id) => id,
                            };

                            ident.to_register().into_compiler_mipsy_result(file_tag.clone(), line, *col, *col_end)?;
                        }
                        MpArgument::Number(_) => {}
                    }
                }
            }
            MpItem::Label(_) => {}
            MpItem::Directive(_) => {}
        }
    }

    // TODO

    Ok(warnings)
}

pub fn check_post_data_label(program: &MpProgram, binary: &Binary) -> MipsyResult<Vec<Warning>> {
    let warnings = vec![];

    for (item, file_tag, line) in program.items() {
        let file_tag = file_tag.clone().unwrap_or_else(|| Rc::from(""));
        let line = *line;

        match item {
            MpItem::Instruction(ref instruction) => {
                for (argument, col, col_end) in instruction.arguments() {
                    match argument {
                        MpArgument::Register(_) => {}
                        MpArgument::Number(number) => {
                            match number {
                                MpNumber::Immediate(imm) => {
                                    match imm {
                                        MpImmediate::LabelReference(label) => {
                                            binary.get_label(label)
                                                .into_compiler_mipsy_result(file_tag.clone(), line, *col, *col_end)?;
                                        }
                                        MpImmediate::I16(_) => {}
                                        MpImmediate::U16(_) => {}
                                        MpImmediate::I32(_) => {}
                                        MpImmediate::U32(_) => {}
                                    }
                                }
                                MpNumber::Float32(_) => {}
                                MpNumber::Float64(_) => {}
                                MpNumber::Char(_) => {}
                            }
                        }
                    }
                }
            }
            MpItem::Label(_) => {}
            MpItem::Directive(_) => {}
        }
    }

    // TODO

    Ok(warnings)
}
