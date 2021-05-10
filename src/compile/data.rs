use std::rc::Rc;

use crate::{KDATA_BOT, KTEXT_BOT, MpProgram, MipsyResult, error::ToMipsyResult, inst::instruction::InstSet, util::Safe};
use super::{
    TEXT_BOT,
    DATA_BOT,
    Binary,
    text::instruction_length,
    bytes::ToBytes
};
use mipsy_parser::{
    MpItem,
    MpDirective,
};

#[derive(PartialEq)]
pub(crate) enum Segment {
    Text,
    Data,
    KText,
    KData,
}

pub fn populate_labels_and_data(binary: &mut Binary, iset: &InstSet, program: &MpProgram) -> MipsyResult<()> {
    let mut text_len = 0;
    let mut ktext_len = 0;
    let mut segment = Segment::Text;

    for (item, file_tag, line) in program.items() {
        let file_tag = file_tag.clone().unwrap_or_else(|| Rc::from(""));
        let line = *line;
        
        match item {
            MpItem::Directive(directive) => {
                // Only allow .text and .data in a Text segment
                if segment == Segment::Text || segment == Segment::KText {
                    match directive {
                        MpDirective::Text | MpDirective::Data | MpDirective::KText | MpDirective::KData => {}
                        _other => {
                            // TODO: WARNING
                        }
                    }
                }

                match directive {
                    MpDirective::Text => segment = Segment::Text,
                    MpDirective::Data => segment = Segment::Data,
                    MpDirective::KText => segment = Segment::KText,
                    MpDirective::KData => segment = Segment::KData,
                    MpDirective::Ascii(string) => {
                        let chars: Vec<char> = string.chars().collect();

                        insert_data(&segment, binary, &chars);
                    }
                    MpDirective::Asciiz(string) => {
                        let chars: Vec<char> = string.chars().collect();

                        insert_data(&segment, binary, &chars);
                        insert_data(&segment, binary, &[0u8]);
                    }
                    MpDirective::Byte(bytes) => {
                        insert_data(&segment, binary, bytes);
                    }
                    MpDirective::Half(halfs) => {
                        insert_data(&segment, binary, halfs);
                    }
                    MpDirective::Word(words) => {
                        insert_data(&segment, binary, words);
                    }
                    MpDirective::Float(floats) => {
                        insert_data(&segment, binary, floats);
                    }
                    MpDirective::Double(doubles) => {
                        insert_data(&segment, binary, doubles);
                    }
                    &MpDirective::Align(num) => {
                        let multiple = 2usize.pow(num);
                        let curr_size = binary.data.len();

                        let num = num as usize;

                        let amount = (num - curr_size) % multiple;
                        if amount < num {
                            // If labels sit before a .align, we want to make them point
                            // at the next aligned value, rather than the padding bytes
                            let mut to_update = vec![];

                            for (label, &addr) in &binary.labels {
                                if addr == TEXT_BOT + (curr_size as u32) {
                                    to_update.push((label.to_string(), addr + (amount as u32)));
                                }
                            }

                            for (label, addr) in to_update {
                                binary.labels.insert(label, addr);
                            }

                            insert_safe_data(&segment, binary, &vec![Safe::Uninitialised; amount]);
                        }
                    }
                    MpDirective::Space(num) => {
                        insert_safe_data(&segment, binary, &vec![Safe::Uninitialised; *num as usize]);
                    }
                    MpDirective::Globl(label) => {
                        binary.globals.push(label.to_string());
                    }
                }
            }
            MpItem::Instruction(instruction) => {
                // We can't compile instructions yet - so just keep track of
                // how many bytes-worth we've seen so far
                match segment {
                    Segment::Text => {
                        text_len += instruction_length(iset, instruction)
                            .to_compiler_mipsy_result(file_tag, line, instruction.col(), instruction.col_end())? * 4;
                    }
                    Segment::KText => {
                        ktext_len += instruction_length(iset, instruction)
                            .to_compiler_mipsy_result(file_tag, line, instruction.col(), instruction.col_end())? * 4;
                    }
                    _ => {
                        todo!()
                    }
                }
            }
            MpItem::Label(label) => {
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
        }
    }

    Ok(())
}

fn insert_data<T: ToBytes>(segment: &Segment, binary: &mut Binary, values: &[T]) {
    insert_safe_data(
        segment,
        binary, 
        &values.iter()
            .flat_map(T::to_bytes)
            .map(Safe::valid)
            .collect::<Vec<Safe<u8>>>()
    );
}

fn insert_safe_data(segment: &Segment, binary: &mut Binary, values: &[Safe<u8>]) {
    match segment {
        Segment::Data  => &mut binary.data,
        Segment::KData => &mut binary.kdata,
        _              => unreachable!()
    }.append(
        &mut values.to_vec()
    );
}
