use std::rc::Rc;

use crate::{CompilerError, KDATA_BOT, KTEXT_BOT, MipsyError, MipsyResult, MpProgram, error::{ToMipsyResult, compiler::{DirectiveType, Error}}, inst::instruction::InstSet, util::Safe};
use super::{
    TEXT_BOT,
    DATA_BOT,
    Binary,
    text::instruction_length,
    bytes::ToBytes
};
use mipsy_parser::{MpArgument, MpConstValue, MpConstValueLoc, MpDirective, MpImmediate, MpItem, MpNumber};
use mipsy_utils::MipsyConfig;

#[derive(PartialEq)]
pub(crate) enum Segment {
    Text,
    Data,
    KText,
    KData,
}

pub(super) fn eval_directive(directive: &MpDirective, binary: &mut Binary, config: &MipsyConfig, file_tag: Rc<str>, segment: &mut Segment, first_pass: bool) -> MipsyResult<Vec<Safe<u8>>> {
    let bytes = match directive {
        MpDirective::Text => {
            *segment = Segment::Text;
            vec![]
        }
        MpDirective::Data => {
            *segment = Segment::Data;
            vec![]
        }
        MpDirective::KText => {
            *segment = Segment::KText;
            vec![]
        }
        MpDirective::KData => {
            *segment = Segment::KData;
            vec![]
        }
        MpDirective::Ascii(ref string) => {
            let chars: Vec<Safe<u8>> = string.chars().flat_map(|c| c.to_bytes()).map(Safe::Valid).collect();

            chars
        }
        MpDirective::Asciiz(string) => {
            let mut chars: Vec<Safe<u8>> = string.chars().flat_map(|c| c.to_bytes()).map(Safe::Valid).collect();
            chars.push(Safe::Valid(0));

            chars
        }
        MpDirective::Byte(bytes) => {
            let mut output = Vec::with_capacity(bytes.len());

            for byte in bytes {
                let value = eval_constant_in_range(&byte, i8::MIN as _, u8::MAX as _, binary, file_tag.clone())?;
                output.push(Safe::Valid(value as u8));
            }

            output
        }
        MpDirective::ByteN(byte, n) => {
            let value  = eval_constant_in_range(&byte, i8::MIN as _,  u8::MAX as _, binary, file_tag.clone())? as u8;
            let amount = eval_constant_in_range(&n,   u32::MIN as _, u32::MAX as _, binary, file_tag.clone())? as u32;
            
            vec![Safe::Valid(value); amount as usize]
        }
        MpDirective::Half(halfs) => {
            let mut output = Vec::with_capacity(halfs.len() * 2);

            for half in halfs {
                let value = eval_constant_in_range(&half, i16::MIN as _, u16::MAX as _, binary, file_tag.clone())?;
                output.append(&mut (value as u16).to_bytes().into_iter().map(Safe::Valid).collect());
            }

            output
        }
        MpDirective::HalfN(half, n) => {
            let value  = eval_constant_in_range(&half, i16::MIN as _, u16::MAX as _, binary, file_tag.clone())? as u16;
            let amount = eval_constant_in_range(&n,    u32::MIN as _, u32::MAX as _, binary, file_tag.clone())? as u32;
            
            let mut output = Vec::with_capacity(amount as usize * 2);

            for _ in 0..amount {
                output.append(&mut value.to_bytes().into_iter().map(Safe::Valid).collect());
            }

            output
        }
        MpDirective::Word(words) => {
            let mut output = Vec::with_capacity(words.len() * 4);

            for word in words {
                let value = eval_constant_in_range(&word, i32::MIN as _, u32::MAX as _, binary, file_tag.clone())?;
                output.append(&mut (value as u32).to_bytes().into_iter().map(Safe::Valid).collect());
            }

            output
        }
        MpDirective::WordN(word, n) => {
            let value  = eval_constant_in_range(&word, i32::MIN as _, u32::MAX as _, binary, file_tag.clone())? as u32;
            let amount = eval_constant_in_range(&n,    u32::MIN as _, u32::MAX as _, binary, file_tag.clone())? as u32;
            
            let mut output = Vec::with_capacity(amount as usize * 4);

            for _ in 0..amount {
                output.append(&mut value.to_bytes().into_iter().map(Safe::Valid).collect());
            }

            output
        }
        MpDirective::Float(floats) => {
            floats.into_iter()
                .flat_map(ToBytes::to_bytes)
                .map(Safe::Valid)
                .collect()
        }
        MpDirective::FloatN(float, n) => {
            let amount = eval_constant_in_range(&n, u32::MIN as _, u32::MAX as _, binary, file_tag.clone())? as u32;

            (0..amount).into_iter()
                .map(|_| float)
                .flat_map(ToBytes::to_bytes)
                .map(Safe::Valid)
                .collect()
        }
        MpDirective::Double(doubles) => {
            doubles.into_iter()
                .flat_map(ToBytes::to_bytes)
                .map(Safe::Valid)
                .collect()
        }
        MpDirective::DoubleN(double, n) => {
            let amount = eval_constant_in_range(&n, u32::MIN as _, u32::MAX as _, binary, file_tag.clone())? as u32;

            (0..amount).into_iter()
                .map(|_| double)
                .flat_map(ToBytes::to_bytes)
                .map(Safe::Valid)
                .collect()
        }
        MpDirective::Align(num) => {
            let num = eval_constant_in_range(&num, u32::MIN as _, 31, binary, file_tag)? as u32;

            let multiple = 2usize.pow(num);
            let curr_size = match segment {
                Segment::Data  => binary.data.len(),
                Segment::KData => binary.kdata.len(),
                Segment::Text  => binary.text.len(),
                Segment::KText => binary.ktext.len(),
            };

            let amount = (multiple - (curr_size % multiple)) % multiple;
            vec![Safe::Uninitialised; amount]
        }
        MpDirective::Space(num) => {
            let num = eval_constant_in_range(&num, u32::MIN as _, u32::MAX as _, binary, file_tag)? as u32;

            let space_byte = if config.spim { Safe::Valid(0) } else { Safe::Uninitialised };
            
            vec![space_byte; num as usize]
        }
        MpDirective::Globl(label) => {
            if first_pass {
                binary.globals.push(label.to_string());
            }

            vec![]
        }
    };

    Ok(bytes)
}

pub fn populate_labels_and_data(binary: &mut Binary, config: &MipsyConfig, iset: &InstSet, program: &mut MpProgram) -> MipsyResult<()> {
    let mut text_len = 0;
    let mut ktext_len = 0;
    let mut segment = Segment::Text;

    for attributed_item in program.items_mut() {
        let line = attributed_item.line_number();
        let file_tag = attributed_item.file_tag()
            .unwrap_or_else(|| Rc::from(""));
        let item = attributed_item.item_mut();
        
        match item {
            MpItem::Directive(directive) => {
                // Only allow .text and .data in a Text segment
                // if segment == Segment::Text || segment == Segment::KText {
                //     match &*directive {
                //         (MpDirective::Text | MpDirective::Data | MpDirective::KText | MpDirective::KData, _) => {}
                //         (other, position) => {
                //             return Err(
                //                 MipsyError::Compiler(
                //                     CompilerError::new(
                //                         Error::DataInTextSegment { directive_type: other.clone() },
                //                         file_tag,
                //                         position.line(),
                //                         position.col(),
                //                         position.col_end(),
                //                     )
                //                 )
                //             );
                //         }
                //     }
                // }

                let bytes = eval_directive(&directive.0, binary, config, file_tag, &mut segment, true)?;
                insert_safe_data(&segment, binary, &bytes);

                match segment {
                    Segment::Text => {
                        text_len += bytes.len();
                    }
                    Segment::KText => {
                        ktext_len += bytes.len();
                    }
                    _ => {}
                }
            }
            MpItem::Instruction(instruction) => {
                for arg in instruction.arguments_mut() {
                    match arg.0 {
                        MpArgument::Number(MpNumber::Immediate(MpImmediate::LabelReference(ref label))) => {
                            if let Some(&value) = binary.constants.get(label) {
                                if u16::MIN <= value as _ && u16::MAX >= value as _ {
                                    arg.0 = MpArgument::Number(MpNumber::Immediate(MpImmediate::U16(value as _)));
                                } else if i16::MIN <= value as _ && i16::MAX >= value as _ {
                                    arg.0 = MpArgument::Number(MpNumber::Immediate(MpImmediate::I16(value as _)));
                                } else if u32::MIN <= value as _ && u32::MAX >= value as _ {
                                    arg.0 = MpArgument::Number(MpNumber::Immediate(MpImmediate::U32(value as _)));
                                } else if i32::MIN <= value as _ && i32::MAX >= value as _ {
                                    arg.0 = MpArgument::Number(MpNumber::Immediate(MpImmediate::I32(value as _)));
                                } else {
                                    todo!();
                                }
                            }
                        },
                        _ => {},
                    }
                }

                // We can't compile instructions yet - so just keep track of
                // how many bytes-worth we've seen so far
                let inst_length = instruction_length(iset, instruction)
                    .into_compiler_mipsy_result(file_tag.clone(), line, instruction.col(), instruction.col_end())? * 4;

                let (bot, length) = match segment {
                    Segment::Text => {
                        (TEXT_BOT, &mut text_len)
                    }
                    Segment::KText => {
                        (KTEXT_BOT, &mut ktext_len)
                    }
                    _ => {
                        return Err(
                            MipsyError::Compiler(
                                CompilerError::new(
                                    Error::InstructionInDataSegment,
                                    file_tag,
                                    instruction.line(),
                                    instruction.col(),
                                    instruction.col_end(),
                                )
                            )
                        );
                    }
                };

                let alignment = (4 - *length % 4) % 4;

                if alignment != 0 {
                    let mut labels = vec![];

                    for (label, &addr) in binary.labels.iter() {
                        if addr as usize == (bot as usize + *length) {
                            labels.push(label.to_string());
                        }
                    }

                    for label in labels {
                        binary.labels.insert(label, bot + (*length + alignment) as u32);
                    }
                }

                *length += alignment + inst_length;
            }
            MpItem::Label(mplabel) => {
                let label = mplabel.label();
                let col = mplabel.col();
                let col_end = mplabel.col_end();

                if binary.labels.contains_key(&*label) {
                    return Err(
                        MipsyError::Compiler(
                            CompilerError::new(
                                Error::RedefinedLabel { label },
                                file_tag,
                                line,
                                col,
                                col_end
                            )
                        )
                    );
                }

                binary.labels.insert(
                    label.to_string(),
                    match segment {
                        Segment::Text => TEXT_BOT + text_len as u32,
                        Segment::Data => DATA_BOT + binary.data.len() as u32,
                        Segment::KText => KTEXT_BOT + ktext_len as u32,
                        Segment::KData => KDATA_BOT + binary.kdata.len() as u32,
                    }
                );
            }
            MpItem::Constant(constant) => {
                let label = constant.label();

                if binary.constants.contains_key(&*label) {
                    return Err(
                        MipsyError::Compiler(
                            CompilerError::new(
                                Error::RedefinedConstant { label: label.to_string() },
                                file_tag,
                                line,
                                constant.col(),
                                constant.col_end()
                            )
                        )
                    );
                }

                let value = eval_constant(binary, constant.value(), file_tag)?;
                binary.constants.insert(label.to_string(), value);
            }
        }
    }

    Ok(())
}

fn eval_constant(binary: &Binary, constant: &MpConstValueLoc, file: Rc<str>) -> MipsyResult<i64> {
    Ok(
        match &constant.0 {
            &MpConstValue::Value(value) => value as _,
            MpConstValue::Const(label) => binary.constants.get(label).copied()
                .ok_or_else(|| MipsyError::Compiler(
                    CompilerError::new(
                        Error::UnresolvedConstant { label: label.to_string() },
                        file.clone(),
                        constant.1.line(),
                        constant.1.col(),
                        constant.1.col_end(),
                    )
                ))?,
            MpConstValue::Minus(value) => -eval_constant(binary, value, file)?,
            MpConstValue::Sum (v1, v2) => eval_constant(binary, v1, file.clone())? + eval_constant(binary, v2, file)?,
            MpConstValue::Sub (v1, v2) => eval_constant(binary, v1, file.clone())? - eval_constant(binary, v2, file)?,
            MpConstValue::Div (v1, v2) => eval_constant(binary, v1, file.clone())? / eval_constant(binary, v2, file)?,
            MpConstValue::Mult(v1, v2) => eval_constant(binary, v1, file.clone())? * eval_constant(binary, v2, file)?,
            MpConstValue::Mod (v1, v2) => eval_constant(binary, v1, file.clone())? % eval_constant(binary, v2, file)?,
            MpConstValue::And (v1, v2) => eval_constant(binary, v1, file.clone())? & eval_constant(binary, v2, file)?,
            MpConstValue::Or  (v1, v2) => eval_constant(binary, v1, file.clone())? | eval_constant(binary, v2, file)?,
            MpConstValue::Xor (v1, v2) => eval_constant(binary, v1, file.clone())? ^ eval_constant(binary, v2, file)?,
            MpConstValue::Neg (value)  => !eval_constant(binary, value, file.clone())?,
            MpConstValue::Shl (v1, v2) => eval_constant(binary, v1, file.clone())? << eval_constant(binary, v2, file)?,
            MpConstValue::Shr (v1, v2) => eval_constant(binary, v1, file.clone())? >> eval_constant(binary, v2, file)?,
        }
    )
}

fn eval_constant_in_range(constant: &MpConstValueLoc, range_low: i64, range_high: i64, binary: &Binary, file: Rc<str>) -> MipsyResult<i64> {
    let value = eval_constant(binary, constant, file.clone())?;

    if value < range_low || value > range_high {
        return Err(MipsyError::Compiler(
            CompilerError::new(
                Error::ConstantValueDoesNotFit {
                    directive_type: DirectiveType::Byte,
                    value,
                    range_low,
                    range_high,
                },
                file,
                constant.1.line(),
                constant.1.col(),
                constant.1.col_end(),
            )
        ));
    }

    Ok(value)
}

fn insert_safe_data(segment: &Segment, binary: &mut Binary, values: &[Safe<u8>]) {
    match segment {
        Segment::Data  => &mut binary.data,
        Segment::KData => &mut binary.kdata,
        // these come later
        Segment::Text | Segment::KText => return,
    }.append(
        &mut values.to_vec()
    );
}
