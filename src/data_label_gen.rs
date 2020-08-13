use crate::context::*;
use crate::gen::*;
use crate::lexer::Token;
use crate::util::*;
use std::convert::TryInto;

pub fn generate_labels_and_data(context: &mut Context) -> GenRes<()> {
    let mut inst = 0;

    while let Some(token) = context.next_useful_token() {
        match token {
            Token::Instruction(inst_name) => {
                inst += get_instruction_length(inst_name);
            }
            Token::Directive(directive) => {
                generate_directive(context, directive)?;
            }
            Token::Label(name) => match context.seg {
                Segment::Text => {
                    context
                        .program
                        .labels
                        .insert(name.to_string(), TEXT_BOT + 4 * inst);
                }
                Segment::Data => {
                    context.program.labels.insert(
                        name.to_string(),
                        DATA_BOT + context.program.data.len() as u32,
                    );
                }
            },
            _ => {
                // ignored
            }
        }
    }

    ok()
}

fn generate_directive(context: &mut Context, directive: &str) -> GenRes<()> {
    match directive {
        "text" => {
            context.seg = Segment::Text;
            ok()
        }
        "data" => {
            context.seg = Segment::Data;
            ok()
        }
        "ascii" => match context.next_useful_token() {
            Some(Token::ConstStr(string)) => {
                ensure_segment(context, Segment::Data)?;

                for chr in string.chars() {
                    context.program.data.push(chr as u8);
                }

                ok()
            }
            other => Err(format!(
                "Expected string after .ascii on line {}, got {:?}",
                context.line, other
            )),
        },
        "asciiz" => match context.next_useful_token() {
            Some(Token::ConstStr(string)) => {
                ensure_segment(context, Segment::Data)?;

                for chr in string.chars() {
                    context.program.data.push(chr as u8);
                }

                // terminating null-byte
                context.program.data.push(0);

                ok()
            }
            other => Err(format!(
                "Expected string after .ascii on line {}, got {:?}",
                context.line, other
            )),
        },
        "byte" => {
            push_data_integer::<u8>(context, |i| i as u8);
            ok()
        }
        "half" => {
            push_data_integer::<u16>(context, |i| i as u16);
            ok()
        }
        "word" => {
            push_data_integer::<u32>(context, |i| i as u32);
            ok()
        }
        "float" => {
            push_data_float::<f32>(context, |i| i as f32);
            ok()
        }
        "double" => {
            push_data_float::<f64>(context, |i| i as f64);
            ok()
        }
        "align" => match context.next_useful_token() {
            Some(&Token::Number(num)) => {
                if num < 0 {
                    return Err(format!(
                        "Expected non-negative number after .align on line {}, got {}",
                        context.line, num
                    ));
                }

                // unwrap ok since num < 0 check
                let multiple = 2usize.pow(num.try_into().unwrap());

                while context.program.data.len() % multiple != 0 {
                    context.program.data.push(0);
                }

                ok()
            }
            other => Err(format!(
                "Expected number after .align on line {}, got {:?}",
                context.line, other
            )),
        },
        "space" => match context.next_useful_token() {
            Some(&Token::Number(num)) => {
                for _ in 0..num {
                    context.program.data.push(0);
                }

                ok()
            }
            other => Err(format!(
                "Expected number after .space on line {}, got {:?}",
                context.line, other
            )),
        },
        "globl" => {
            // handled later - can't resolve label reference yet.
            ok()
        }
        _ => Err(format!(
            "Unknown directive on line {}: .{}",
            context.line, directive
        )),
    }
}

fn push_data_integer<T: ToMipsBytes>(context: &mut Context, f: fn(i32) -> T) {
    while let Some(token) = context.peek_useful_token() {
        match *token {
            Token::Number(num) => {
                context.program.data.append(&mut f(num).to_mips_bytes());

                context.next_token();
            }
            _ => {
                break;
            }
        }
    }
}

fn push_data_float<T: ToMipsBytes>(context: &mut Context, f: fn(f64) -> T) {
    while let Some(token) = context.peek_useful_token() {
        match *token {
            Token::Float(num) => {
                context.program.data.append(&mut f(num).to_mips_bytes());

                context.next_token();
            }
            _ => {
                break;
            }
        }
    }
}

fn get_instruction_length(inst: &str) -> u32 {
    match inst {
        // TODO add all pseudoinstructions
        // TODO check all these
        "abs" => 3,
        "blt" => 2,
        "bgt" => 2,
        "ble" => 2,
        "neg" => 2,
        "not" => 2,
        "bge" => 2,
        "li" => 2,
        "la" => 2,
        "move" => 1,
        "sge" => 2,
        "sgt" => 2,
        _ => 1,
    }
}

fn ensure_segment(context: &mut Context, seg: Segment) -> GenRes<()> {
    if context.seg == seg {
        ok()
    } else {
        Err(format!(
            "Incorrect segment on line {}: Expected segment {}, but was in segment {}",
            context.line, seg, context.seg
        ))
    }
}

trait ToMipsBytes {
    fn to_mips_bytes(&self) -> Vec<u8>;
}

impl ToMipsBytes for u8 {
    fn to_mips_bytes(&self) -> Vec<u8> {
        vec![*self]
    }
}

impl ToMipsBytes for u16 {
    fn to_mips_bytes(&self) -> Vec<u8> {
        if LITTLE_ENDIAN {
            self.to_le_bytes()
        } else {
            self.to_be_bytes()
        }
        .to_vec()
    }
}

impl ToMipsBytes for u32 {
    fn to_mips_bytes(&self) -> Vec<u8> {
        if LITTLE_ENDIAN {
            self.to_le_bytes()
        } else {
            self.to_be_bytes()
        }
        .to_vec()
    }
}

impl ToMipsBytes for f32 {
    fn to_mips_bytes(&self) -> Vec<u8> {
        if LITTLE_ENDIAN {
            self.to_le_bytes()
        } else {
            self.to_be_bytes()
        }
        .to_vec()
    }
}

impl ToMipsBytes for f64 {
    fn to_mips_bytes(&self) -> Vec<u8> {
        if LITTLE_ENDIAN {
            self.to_le_bytes()
        } else {
            self.to_be_bytes()
        }
        .to_vec()
    }
}
