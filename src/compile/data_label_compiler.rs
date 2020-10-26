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
use crate::inst::pseudo::PseudoInst;
use crate::util::Safe;


pub fn generate_labels_and_data(context: &mut Context, iset: &InstSet) -> RSpimResult<()> {
    let mut inst: usize = 0;

    while let Some(token) = context.next_useful_token() {
        match token {
            Token::Instruction(inst_name) => {
                inst += get_instruction_length(context, iset, inst_name)?;
            }
            Token::Directive(directive) => {
                generate_directive(context, directive)?;
            }
            Token::Label(name) => match context.seg {
                Segment::Text => {
                    context
                        .program
                        .labels
                        .insert(name.to_string(), TEXT_BOT + 4 * inst as u32);
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

fn align(context: &mut Context, multiple: usize) {
    let mut extra = 0;

    let mut labels = vec![];

    for (label, &addr) in &context.program.labels {
        if addr == DATA_BOT + context.program.data.len() as u32 {
            labels.push((label.clone(), addr));
        }
    }

    while context.program.data.len() % multiple != 0 {
        extra += 1;
        context.program.data.push(Safe::Uninitialised);
    }

    if extra > 0 {
        for (label, addr) in labels {
            context.program.labels.insert(label, addr + extra);
        }
    }
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
                    context.program.data.push(Safe::Valid(chr as u8));
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
                    context.program.data.push(Safe::Valid(chr as u8));
                }

                // terminating null-byte
                context.program.data.push(Safe::Valid(0));

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
            push_data_integer::<u8>(context, |i| i as u8, |i| i as u8);
            ok()
        }
        "half" => {
            align(context, 2);
            push_data_integer::<u16>(context, |i| i as u16, |i| i as u16);
            ok()
        }
        "word" => {
            align(context, 4);
            push_data_integer::<u32>(context, |i| i as u32, |i| i as u32);
            ok()
        }
        "float" => {
            align(context, 4);
            push_data_float::<f32>(context, |i| i as f32);
            ok()
        }
        "double" => {
            align(context, 8);
            push_data_float::<f64>(context, |i| i as f64);
            ok()
        }
        "align" => {
            let num = match context.next_useful_token() {
                Some(&Token::Immediate(num)) => num as i32,
                Some(&Token::Word(num)) => num,
                other => return cerr!(
                    CompileError::CompilerAlignExpectedNum {
                        line: context.line,
                        got_instead: other.unwrap_or(&Token::EOF).clone()
                    }
                ),
            };
            
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

            align(context, multiple);
            ok()
        }
        "space" => {
            let num = match context.next_useful_token() {
                Some(&Token::Immediate(num)) => num as i32,
                Some(&Token::Word(num)) => num,
                other => return cerr!(
                    CompileError::CompilerAlignExpectedNum {
                        line: context.line,
                        got_instead: other.unwrap_or(&Token::EOF).clone()
                    }
                ),
            };

            if num < 0 {
                return cerr!(
                    CompileError::CompilerSpaceExpectedPos {
                        line: context.line,
                        got_instead: num as i32,
                    }
                );
            }

            for _ in 0..num {
                context.program.data.push(Safe::Uninitialised);
            }

            ok()
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

fn push_data_integer<T: ToMipsBytes>(context: &mut Context, f: fn(i32) -> T, g: fn(i16) -> T) {
    while let Some(token) = context.peek_useful_token() {
        match *token {
            Token::Word(num) => {
                context.program.data.append(&mut f(num).to_mips_bytes().iter().map(|&b| Safe::Valid(b)).collect());

                context.next_token();
            }
            Token::Immediate(num) => {
                context.program.data.append(&mut g(num).to_mips_bytes().iter().map(|&b| Safe::Valid(b)).collect());

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
                context.program.data.append(&mut f(num).to_mips_bytes().iter().map(|&b| Safe::Valid(b)).collect());

                context.next_token();
            }
            _ => {
                break;
            }
        }
    }
}

fn get_instruction_length(context: &Context, iset: &InstSet, inst_name: &str) -> RSpimResult<usize> {
    let (inst, _) = super::text_compiler::find_instruction(&inst_name.to_ascii_lowercase(), &mut context.clone(), iset)?;

    let length = match inst {
        super::text_compiler::ParsedInst::Native(_) => 1,
        super::text_compiler::ParsedInst::Pseudo(p) => p.len(context),
    };

    Ok(length)
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
