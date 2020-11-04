use crate::error::RSpimResult;
use crate::error::RSpimError;
use crate::error::CompileError;
use crate::cerr;
use super::context::Context;
use super::context::Token;
use super::compiler::TEXT_BOT;
use crate::inst::instruction::{InstSet, InstSignature, PseudoSignature};
use crate::inst::instruction::GenericSignature;
use crate::inst::pseudo::PseudoInst;

pub enum ParsedInst<'a> {
    Native(&'a InstSignature),
    Pseudo(&'a PseudoSignature),
}

pub fn find_instruction<'a, 'b>(name: &str, context: &'b mut Context, iset: &'a InstSet) -> RSpimResult<(ParsedInst<'a>, Vec<Token>)> {
    let name = name.to_ascii_lowercase();
    
    let mut poss_native = vec![];
    let mut poss_pseudo = vec![];

    for inst in &iset.native_set {
        if inst.name == name {
            poss_native.push(inst);
        }
    }

    for inst in &iset.pseudo_set {
        if inst.name == name {
            poss_pseudo.push(inst);
        }
    }

    if poss_native.is_empty() && poss_pseudo.is_empty() {
        return cerr!(CompileError::UnknownInstruction(name.into()));
    }

    let mut arg_tokens: Vec<Token> = vec![];

    let mut comma = true;
    loop {
        if let Some(token) = context.peek_useful_token() {
            match token {
                Token::Register(_) | Token::Word(_) | Token::Immediate(_) | Token::Float(_)
                    | Token::LabelReference(_) | Token::OffsetRegister(_) | Token::ConstChar(_) => {
                    if !comma {
                        return cerr!(CompileError::MissingComma);
                    }

                    if matches!(token, Token::OffsetRegister(_)) {
                        if arg_tokens.is_empty() || !matches!(arg_tokens.last().unwrap(), Token::Word(_) | Token::Immediate(_) | Token::LabelReference(_)) {
                            arg_tokens.push(Token::Immediate(0));
                        }
                    }

                    arg_tokens.push(token)
                }
                Token::Comma => {
                    if comma {
                        return cerr!(CompileError::UnexpectedComma)
                    }

                    comma = true;
                }
                _ => break,
            }
        } else {
            break;
        }

        context.next_useful_token();
    }

    poss_native.retain(|inst| inst.compile.tokens_match(&arg_tokens));
    poss_pseudo.retain(|inst| inst.compile.tokens_match(&arg_tokens));

    if poss_native.is_empty() && poss_pseudo.is_empty() {
        return cerr!(CompileError::InstructionBadFormat(name.into()));
    }

    if (!poss_native.is_empty() && !poss_pseudo.is_empty()) ||
        poss_native.len() > 1 || poss_pseudo.len() > 1 {
        return cerr!(CompileError::MultipleMatchingInstructions(
            poss_native.iter().map(|&inst| GenericSignature::Native(inst.clone()))
                .chain(
                    poss_pseudo.iter().map(|&inst| GenericSignature::Pseudo(inst.clone()))
                )
                .collect()
        ));
    }

    Ok((
        if poss_native.is_empty() {
            ParsedInst::Pseudo(poss_pseudo[0])
        } else {
            ParsedInst::Native(poss_native[0])
        }, 

        arg_tokens
    ))
}

pub fn parse_instruction(name: &str, context: &mut Context, iset: &InstSet) -> RSpimResult<Vec<u32>> {
    let mut context_clone = context.clone();

    let (inst, arg_tokens) = find_instruction(name, &mut context_clone, iset)?;

    let (format, len) = 
        match inst {
            ParsedInst::Native(n) => (&n.compile, 1),
            ParsedInst::Pseudo(p) => (&p.compile, p.len(context)),
        };

    let mut input: Vec<u32> = vec![];

    let arg_tokens_len = arg_tokens.len();
    for token in arg_tokens {
        match token {
            Token::Register(reg) => {
                input.push(crate::inst::register::Register::from_str(&reg)?.to_number() as u32);
            }
            Token::Immediate(num) => {
                input.push(num as u32);
            }
            // TODO
            Token::Word(num) => {
                input.push(num as u32);
            }
            Token::Float(_flt) => {
                unimplemented!()
            }
            Token::ConstChar(chr) => {
                input.push(chr as u32);
            }
            Token::LabelReference(ref label) => {
                let addr = *context.program.labels.get(label)
                    .ok_or_else(||
                        RSpimError::Compile(
                            CompileError::UnresolvedLabel(label.to_string())
                        )
                    )?;

                let value =                       // only final label applies
                    if format.relative_label && input.len() == arg_tokens_len - 1 {

                        let current_inst_addr = (context.program.text.len() + len - 1) as u32 * 4 + TEXT_BOT;

                        (addr.wrapping_sub(current_inst_addr)) / 4
                    } else {
                        addr
                    };

                input.push(value);
            }
            Token::OffsetRegister(oreg) => {
                input.push(crate::inst::register::Register::from_str(&oreg)?.to_number() as u32);
            }
            _ => unreachable!(),
        }
    }

    match inst {
        ParsedInst::Native(n) => Ok(vec![n.gen_op(&input)?]),
        ParsedInst::Pseudo(p) => Ok(p.expand(iset, &input)?),
    }
}

pub fn generate_text(context: &mut Context, iset: &InstSet) -> RSpimResult<()> {
    let mut text = true;

    while let Some(token) = context.next_useful_token() {
        match token {
            Token::Instruction(ref name) => {
                if !text {
                    return cerr!(CompileError::InstructionInDataSegment);
                }

                let parsed = &mut parse_instruction(name, context, iset)?;

                context.program.text.append(parsed);
            }
            Token::Directive(directive) => {
                match &directive.to_ascii_lowercase() as &str {
                    "text" => text = true,
                    "data" => text = false,
                    _ => {}
                }
            }
            _ => {}
        }
    }
    
    Ok(())
}