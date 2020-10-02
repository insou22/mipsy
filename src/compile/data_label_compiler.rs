use super::context::Token;
use super::context::Context;
use super::compiler::DATA_BOT;
use super::compiler::TEXT_BOT;
use super::context::Segment;
use crate::error::RSpimResult;
use crate::error::CompileError;
use crate::cerr;
use std::convert::TryInto;
use crate::inst::instruction::InstSet;


pub fn generate_labels_and_data(context: &mut Context, iset: &InstSet) -> RSpimResult<()> {
    let mut inst = 0;

    while let Some(token) = context.next_useful_token() {
        match token {
            Token::Instruction(inst_name) => {
                inst += get_instruction_length(context, iset, inst_name);
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

fn generate_directive(context: &mut Context, directive: &str) -> RSpimResult<()> {
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
            other => cerr!(
                CompileError::CompilerAsciiExpectedString { 
                    line: context.line, 
                    got_instead: other.unwrap_or(&Token::EOF).clone()
                } 
            ),
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
            other => cerr!(
                CompileError::CompilerAsciiExpectedString { 
                    line: context.line, 
                    got_instead: other.unwrap_or(&Token::EOF).clone()
                } 
            ),
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
                    return cerr!(
                        CompileError::CompilerAlignExpectedPos {
                            line: context.line,
                            got_instead: num,
                        }
                    );
                }

                // unwrap ok since num < 0 check
                let multiple = 2usize.pow(num.try_into().unwrap());

                while context.program.data.len() % multiple != 0 {
                    context.program.data.push(0);
                }

                ok()
            }
            other => cerr!(
                CompileError::CompilerAlignExpectedNum {
                    line: context.line,
                    got_instead: other.unwrap_or(&Token::EOF).clone()
                }
            ),
        },
        "space" => match context.next_useful_token() {
            Some(&Token::Number(num)) => {
                if num < 0 {
                    return cerr!(
                        CompileError::CompilerSpaceExpectedPos {
                            line: context.line,
                            got_instead: num,
                        }
                    );
                }

                for _ in 0..num {
                    context.program.data.push(0);
                }

                ok()
            }
            other => cerr!(
                CompileError::CompilerSpaceExpectedNum {
                    line: context.line,
                    got_instead: other.unwrap_or(&Token::EOF).clone()
                }
            ),
        },
        "globl" => {
            // handled later - can't resolve label reference yet.
            ok()
        }
        _ => cerr!(CompileError::CompilerUnknownDirective {
            line: context.line,
            got_instead: directive.into(),
        }),
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

fn get_instruction_length(context: &Context, iset: &InstSet, inst: &str) -> u32 {
    let _context = context.clone();

    for pseudo in &iset.pseudo_set {
        if pseudo.name == inst.to_ascii_lowercase() {
            
        }
    }

    1
}

fn ensure_segment(context: &mut Context, seg: Segment) -> RSpimResult<()> {
    if context.seg == seg {
        ok()
    } else {
        cerr!(CompileError::CompilerIncorrectSegment {
            line: context.line,
            current_segment: context.seg,
            needed_segment: seg
        })
    }
}

fn ok<T: Default>() -> RSpimResult<T> {
    Ok(T::default())
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
        self.to_le_bytes().to_vec()
    }
}

impl ToMipsBytes for u32 {
    fn to_mips_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ToMipsBytes for f32 {
    fn to_mips_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl ToMipsBytes for f64 {
    fn to_mips_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}
