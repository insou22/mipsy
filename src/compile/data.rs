use crate::{
    cerr,
    RSpimResult,
    MPProgram,
    error::CompileError,
    util::Safe,
    inst::instruction::InstSet,
};
use super::{
    TEXT_BOT,
    DATA_BOT,
    Binary,
    text::instruction_length,
    bytes::ToBytes
};
use rspim_parser::{
    MPItem,
    MPDirective,
};

#[derive(PartialEq)]
enum Segment {
    Text,
    Data,
}

pub fn populate_labels_and_data(binary: &mut Binary, iset: &InstSet, program: &MPProgram) -> RSpimResult<()> {
    let mut text_len = 0;
    let mut segment = Segment::Text;

    for item in program.items() {
        match item {
            MPItem::Directive(directive) => {
                // Only allow .text and .data in a Text segment
                if segment == Segment::Text {
                    match directive {
                        MPDirective::Text | MPDirective::Data => {}
                        other => return cerr!(CompileError::String(format!("Directive in Text segment [Directive={:?}]", other))),
                    }
                }

                match directive {
                    MPDirective::Text => segment = Segment::Text,
                    MPDirective::Data => segment = Segment::Data,
                    MPDirective::Ascii(string) => {
                        let chars: Vec<char> = string.chars().collect();
                        
                        insert_data(binary, &chars);
                    }
                    MPDirective::Asciiz(string) => {
                        let chars: Vec<char> = string.chars().collect();

                        insert_data(binary, &chars);
                        insert_data(binary, &[0]);
                    }
                    MPDirective::Byte(bytes) => {
                        insert_data(binary, bytes);
                    }
                    MPDirective::Half(halfs) => {
                        insert_data(binary, halfs);
                    }
                    MPDirective::Word(words) => {
                        insert_data(binary, words);
                    }
                    MPDirective::Float(floats) => {
                        insert_data(binary, floats);
                    }
                    MPDirective::Double(doubles) => {
                        insert_data(binary, doubles);
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

                            insert_safe_data(binary, &vec![Safe::Uninitialised; amount]);
                        }
                    }
                    MPDirective::Space(num) => {
                        insert_safe_data(binary, &vec![Safe::Uninitialised; *num as usize]);
                    }
                    MPDirective::Globl(label) => {
                        binary.globals.push(label.to_string());
                    }
                }
            }
            MPItem::Instruction(instruction) => {
                // We can't compile instructions yet - so just keep track of
                // how many bytes-worth we've seen so far
                text_len += instruction_length(iset, instruction)? * 4;
            }
            MPItem::Label(label) => {
                binary.labels.insert(
                    label.to_string(),
                    match segment {
                        Segment::Text => TEXT_BOT + text_len as u32,
                        Segment::Data => DATA_BOT + binary.data.len() as u32,
                    }
                );
            }
        }
    }

    Ok(())
}

fn insert_data<T: ToBytes>(binary: &mut Binary, values: &[T]) {
    insert_safe_data(
        binary, 
        &values.iter()
            .flat_map(T::to_bytes)
            .map(Safe::valid)
            .collect::<Vec<Safe<u8>>>()
    );
}

fn insert_safe_data(binary: &mut Binary, values: &[Safe<u8>]) {
    binary.data.append(
        &mut values.to_vec()
    );
}
