use std::rc::Rc;

use mipsy_parser::{MpArgument, MpImmediate, MpItem, MpNumber};

use crate::{
    error::{compiler, ToMipsyResult},
    inst::instruction::ToRegister,
    Binary, CompilerError, MipsyError, MipsyResult, MpProgram, DATA_BOT, HEAP_BOT,
};

pub enum Warning {}

pub fn check_pre(program: &MpProgram) -> MipsyResult<Vec<Warning>> {
    let warnings = vec![];

    for attributed_item in program.items() {
        let item = attributed_item.item();
        let line = attributed_item.line_number();
        let file_tag = attributed_item.file_tag().unwrap_or_else(|| Rc::from(""));

        match item {
            MpItem::Instruction(ref instruction) => {
                for (argument, col, col_end) in instruction.arguments() {
                    match argument {
                        MpArgument::Register(register) => {
                            let ident = register.get_identifier();
                            ident.to_register().into_compiler_mipsy_result(
                                file_tag.clone(),
                                line,
                                *col,
                                *col_end,
                            )?;
                        }
                        MpArgument::Number(_) => {}
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
        let file_tag = attributed_item.file_tag().unwrap_or_else(|| Rc::from(""));

        match item {
            MpItem::Instruction(ref instruction) => {
                for (argument, col, col_end) in instruction.arguments() {
                    if let MpArgument::Number(MpNumber::Immediate(imm)) = argument {
                        check_imm(binary, imm, file_tag.clone(), line, *col, *col_end)?
                    }
                }
            }
            MpItem::Label(_) => {}
            MpItem::Directive(_) => {}
            MpItem::Constant(_) => {}
        }
    }

    if binary.data.len() > (HEAP_BOT - DATA_BOT) as usize {
        return Err(MipsyError::Compiler(CompilerError::new(
            compiler::Error::TooMuchData {
                data_size: binary.data.len() as u32,
            },
            // this all doesn't end up being used
            Rc::from(""),
            0,
            0,
            0,
        )));
    }

    // TODO

    Ok(warnings)
}

fn check_imm(
    binary: &Binary,
    imm: &MpImmediate,
    file_tag: Rc<str>,
    line: u32,
    col: u32,
    col_end: u32,
) -> MipsyResult<()> {
    match imm {
        MpImmediate::LabelReference(label) => {
            if binary.constants.get(label).is_none() {
                binary
                    .get_label(label)
                    .into_compiler_mipsy_result(file_tag, line, col, col_end)?;
            }
        }
        MpImmediate::I16(_) | MpImmediate::U16(_) | MpImmediate::I32(_) | MpImmediate::U32(_) => {}
    }

    Ok(())
}
