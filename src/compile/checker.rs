use mipsy_parser::{MPArgument, MPImmediate, MPItem, MPNumber, MPRegister};

use crate::{Binary, MPProgram, MipsyResult, inst::instruction::ToRegister, util::WithLoc};

pub enum Warning {

}

pub fn check_pre(program: &MPProgram) -> MipsyResult<Vec<Warning>> {
    let warnings = vec![];

    for item in program.items() {
        let line = item.1;

        match &item.0 {
            MPItem::Instruction(ref instruction) => {
                for (argument, col, col_end) in instruction.arguments() {
                    match argument {
                        MPArgument::Register(register) => {
                            let ident = match register {
                                MPRegister::Normal(id) => id,
                                MPRegister::Offset(_, id) => id,
                            };

                            ident.to_register().with_line(line).with_col(*col).with_col_end(*col_end)?;
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

    for item in program.items() {
        let line = item.1;

        match &item.0 {
            MPItem::Instruction(ref instruction) => {
                for (argument, col, col_end) in instruction.arguments() {
                    match argument {
                        MPArgument::Register(_) => {}
                        MPArgument::Number(number) => {
                            match number {
                                MPNumber::Immediate(imm) => {
                                    match imm {
                                        MPImmediate::LabelReference(label) => {
                                            binary.get_label(label).with_line(line).with_col(*col).with_col_end(*col_end)?;
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
