use std::rc::Rc;

use mipsy_parser::{MpArgument, MpImmediate, MpItem, MpNumber};

use crate::{Binary, MpProgram, MipsyResult, error::ToMipsyResult, inst::instruction::ToRegister};

pub enum Warning {

}

pub fn check_pre(program: &MpProgram) -> MipsyResult<Vec<Warning>> {
    let warnings = vec![];

    for attributed_item in program.items() {
        let item = attributed_item.item();
        let line = attributed_item.line_number();
        let file_tag = attributed_item.file_tag()
            .unwrap_or_else(|| Rc::from(""));

        match item {
            MpItem::Instruction(ref instruction) => {
                for (argument, col, col_end) in instruction.arguments() {
                    match argument {
                        MpArgument::Register(register) => {
                            let ident = register.get_identifier();
                            ident.to_register().into_compiler_mipsy_result(file_tag.clone(), line, *col, *col_end)?;
                        }
                        MpArgument::Number(_) => {}
                        // MpArgument::LabelPlusConst(..) => {}
                    }
                }
            }
            MpItem::Label(_) => {}
            MpItem::Directive(_) => {}
            MpItem::Constant(_) => {}
        }
    }

    // TODO

    Ok(warnings)
}

pub fn check_post_data_label(program: &MpProgram, binary: &Binary) -> MipsyResult<Vec<Warning>> {
    let warnings = vec![];

    for attributed_item in program.items() {
        let item = attributed_item.item();
        let line = attributed_item.line_number();
        let file_tag = attributed_item.file_tag()
            .unwrap_or_else(|| Rc::from(""));

        match item {
            MpItem::Instruction(ref instruction) => {
                for (argument, col, col_end) in instruction.arguments() {
                    match argument {
                        MpArgument::Register(_) => {}
                        MpArgument::Number(number) => {
                            match number {
                                MpNumber::Immediate(imm) => check_imm(binary, imm, file_tag.clone(), line, *col, *col_end)?,
                                MpNumber::BinaryOpImmediate(i1, _, i2) => {
                                    check_imm(binary, i1, file_tag.clone(), line, *col, *col_end)?;
                                    check_imm(binary, i2, file_tag.clone(), line, *col, *col_end)?;
                                }
                                MpNumber::Float32(_) => {}
                                MpNumber::Float64(_) => {}
                                MpNumber::Char(_) => {}
                            }
                        }
                        // MpArgument::LabelPlusConst(label, _const) => {
                        //     if binary.constants.get(label).is_none() {
                        //         binary.get_label(label)
                        //             .into_compiler_mipsy_result(file_tag.clone(), line, *col, *col_end)?;
                        //     }
                        // }
                    }
                }
            }
            MpItem::Label(_) => {}
            MpItem::Directive(_) => {}
            MpItem::Constant(_) => {}
        }
    }

    // TODO

    Ok(warnings)
}

fn check_imm(binary: &Binary, imm: &MpImmediate, file_tag: Rc<str>, line: u32, col: u32, col_end: u32) -> MipsyResult<()> {
    match imm {
        MpImmediate::LabelReference(label) => {
            if binary.constants.get(label).is_none() {
                binary.get_label(label)
                    .into_compiler_mipsy_result(file_tag, line, col, col_end)?;
            }
        }
        MpImmediate::I16(_)
        | MpImmediate::U16(_)
        | MpImmediate::I32(_)
        | MpImmediate::U32(_) => {}
    }

    Ok(())
}
