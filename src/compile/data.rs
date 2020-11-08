use crate::{KDATA_BOT, KTEXT_BOT, MPProgram, MipsyResult, cerr, error::CompileError, inst::instruction::InstSet, util::Safe};
use super::{
    TEXT_BOT,
    DATA_BOT,
    Binary,
    text::instruction_length,
    bytes::ToBytes
};
use mipsy_parser::{
    MPItem,
    MPDirective,
};

#[derive(PartialEq)]
pub(crate) enum Segment {
    Text,
    Data,
    KText,
    KData,
}

pub fn populate_labels_and_data(binary: &mut Binary, iset: &InstSet, program: &MPProgram) -> MipsyResult<()> {
    let mut text_len = 0;
    let mut ktext_len = 0;
    let mut segment = Segment::Text;

    for item in program.items() {
        match item {
            MPItem::Directive(directive) => {
                // Only allow .text and .data in a Text segment
                if segment == Segment::Text || segment == Segment::KText {
                    match directive {
                        MPDirective::Text | MPDirective::Data | MPDirective::KText | MPDirective::KData => {}
                        other => return cerr!(CompileError::String(format!("Directive in Text segment [Directive={:?}]", other))),
                    }
                }

                match directive {
                    MPDirective::Text => segment = Segment::Text,
                    MPDirective::Data => segment = Segment::Data,
                    MPDirective::KText => segment = Segment::KText,
                    MPDirective::KData => segment = Segment::KData,
                    MPDirective::Ascii(string) => {
                        let chars: Vec<char> = string.chars().collect();

                        insert_data(&segment, binary, &chars);
                    }
                    MPDirective::Asciiz(string) => {
                        let chars: Vec<char> = string.chars().collect();

                        insert_data(&segment, binary, &chars);
                        insert_data(&segment, binary, &[0]);
                    }
                    MPDirective::Byte(bytes) => {
                        insert_data(&segment, binary, bytes);
                    }
                    MPDirective::Half(halfs) => {
                        insert_data(&segment, binary, halfs);
                    }
                    MPDirective::Word(words) => {
                        insert_data(&segment, binary, words);
                    }
                    MPDirective::Float(floats) => {
                        insert_data(&segment, binary, floats);
                    }
                    MPDirective::Double(doubles) => {
                        insert_data(&segment, binary, doubles);
                    }
                    &MPDirective::Align(num) => {
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
                    MPDirective::Space(num) => {
                        insert_safe_data(&segment, binary, &vec![Safe::Uninitialised; *num as usize]);
                    }
                    MPDirective::Globl(label) => {
                        binary.globals.push(label.to_string());
                    }
                }
            }
            MPItem::Instruction(instruction) => {
                // We can't compile instructions yet - so just keep track of
                // how many bytes-worth we've seen so far
                match segment {
                    Segment::Text => {
                        text_len += instruction_length(iset, instruction)? * 4;
                    }
                    Segment::KText => {
                        ktext_len += instruction_length(iset, instruction)? * 4;
                    }
                    _ => {
                        todo!()
                    }
                }
            }
            MPItem::Label(label) => {
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
