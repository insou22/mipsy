use crate::{
    cerr,
    RSpimResult,
    MPProgram,
    error::CompileError,
    util::Safe,
};
use super::{
    TEXT_BOT,
    DATA_BOT,
    Context,
    Segment,
    text::instruction_length,
    bytes::ToBytes
};
use rspim_parser::{
    MPItem,
    MPDirective,
};

pub fn populate_labels_and_data(context: &mut Context, program: &MPProgram) -> RSpimResult<()> {
    let text_len = 0;
    let data_len = 0;

    for item in program.items() {
        match item {
            MPItem::Directive(directive) => {
                // Only allow .text and .data in a Text segment
                if context.segment == Segment::Text {
                    match directive {
                        MPDirective::Text => {}
                        MPDirective::Data => {}
                        _ => return cerr!(CompileError::Str("Directive in Text segment")),
                    }
                }

                match directive {
                    MPDirective::Text => context.segment = Segment::Text,
                    MPDirective::Data => context.segment = Segment::Data,
                    MPDirective::Ascii(string) => {
                        let chars: Vec<char> = string.chars().collect();
                        
                        insert_data(context, &chars);
                    }
                    MPDirective::Asciiz(string) => {
                        let chars: Vec<char> = string.chars().collect();

                        insert_data(context, &chars);
                        insert_data(context, &[0]);
                    }
                    MPDirective::Byte(bytes) => {
                        insert_data(context, bytes);
                    }
                    MPDirective::Half(halfs) => {
                        insert_data(context, halfs);
                    }
                    MPDirective::Word(words) => {
                        insert_data(context, words);
                    }
                    MPDirective::Float(floats) => {
                        insert_data(context, floats);
                    }
                    MPDirective::Double(doubles) => {
                        insert_data(context, doubles);
                    }
                    &MPDirective::Align(num) => {
                        let multiple = 2usize.pow(num);
                        let curr_size = context.binary.data.len();

                        let num = num as usize;

                        let mut amount = num - curr_size % num;
                        if amount < num {
                            // If labels sit before a .align, we want to make them point
                            // at the next aligned value, rather than the padding bytes
                            for (label, addr) in context.binary.labels {
                                if addr == TEXT_BOT + (curr_size as u32) {
                                    context.binary.labels.insert(label, addr + (amount as u32));
                                }
                            }

                            insert_safe_data(context, &vec![Safe::Uninitialised; amount]);
                        }
                    }
                    MPDirective::Space(num) => {
                        insert_safe_data(context, &vec![Safe::Uninitialised; *num as usize]);
                    }
                    MPDirective::Globl(label) => {
                        context.binary.globals.push(*label);
                    }
                }
            }
            MPItem::Instruction(instruction) => {
                // We can't compile instructions yet - so just keep track of
                // how many bytes-worth we've seen so far
                context.unmapped_text_len += instruction_length(instruction)? * 4;
            }
            MPItem::Label(label) => {
                context.binary.labels.insert(
                    *label,
                    match context.segment {
                        Segment::Text => TEXT_BOT + context.unmapped_text_len as u32,
                        Segment::Data => DATA_BOT + context.binary.data.len() as u32,
                    }
                );
            }
        }
    }

    Ok(())
}

fn insert_data<T: ToBytes>(context: &mut Context, values: &[T]) {
    insert_safe_data(
        context, 
        &values.iter()
            .flat_map(T::to_bytes)
            .map(Safe::valid)
            .collect::<Vec<Safe<u8>>>()
    );
}

fn insert_safe_data(context: &mut Context, values: &[Safe<u8>]) {
    context.binary.data.append(
        &mut values.iter().cloned().collect()
    );
}
