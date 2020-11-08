use crate::{
    directive::{
        MPDirective,
        parse_directive,
    },
    instruction::{
        MPInstruction,
        parse_instruction,
    },
    label::parse_label,
    misc::comment_multispace0,
};
use nom::{
    IResult,
    sequence::tuple,
    combinator::map,
    multi::many0,
    branch::alt,
};


#[derive(Debug, Clone, PartialEq)]
pub struct MPProgram {
    pub(crate) items: Vec<MPItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MPItem {
    Instruction(MPInstruction),
    Directive(MPDirective),
    Label(String),
}

impl MPProgram {
    pub fn items(&self) -> &[MPItem] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Vec<MPItem> {
        &mut self.items
    }
}

pub fn parse_mips_item(i: &[u8]) -> IResult<&[u8], MPItem> {
    map(
        tuple((
            comment_multispace0,
            alt((
                map(parse_label,       |l| MPItem::Label(l)),
                map(parse_directive,   |d| MPItem::Directive(d)),
                map(parse_instruction, |i| MPItem::Instruction(i)),
            )),
            comment_multispace0,
        )),
        |(_, item, _)| item 
    )(i)
}

pub fn parse_mips_bytes(i: &[u8]) -> IResult<&[u8], MPProgram> {
    let (
        remaining_input,
        items
    ) = many0(parse_mips_item)(i)?;

    Ok((
        remaining_input,
        MPProgram {
            items
        },
    ))
}

pub fn parse_mips<T>(input: T) -> Result<MPProgram, &'static str>
where
    T: AsRef<str>,
{
    match parse_mips_bytes(input.as_ref().trim().as_bytes()) {
        Ok((rest, program)) => {
            if rest.len() == 0 {
                Ok(program)
            } else {
                println!("WARNING: file had parse leftovers:\n{}", String::from_utf8_lossy(rest).to_string());

                Ok(program)
            }
        },
        Err(_) => Err("Failed to parse"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MPItem::*;
    use crate::MPArgument::*;
    use crate::MPRegister::*;
    use crate::MPRegisterIdentifier::*;
    use crate::MPNumber::*;
    use crate::MPImmediate::*;

    #[test]
    fn add_s() {
        assert_eq!(
            parse_mips("
                # add 17 and 25  and print result

                main:                    #  x, y, z in $t0, $t1, $t2,
                    li   $t0, 17         # x = 17;

                    li   $t1, 25         # y = 25;

                    add  $t2, $t1, $t0   # z = x + y

                    move $a0, $t2        # printf(\"%d\", a0);
                    li   $v0, 1
                    syscall

                    li   $a0, '\\n'       # printf(\"%c\", '\\n');
                    li   $v0, 11
                    syscall

                    li   $v0, 0          # return 0
                    jr   $ra

            ").unwrap(),
            MPProgram {
                items: vec![
                    Label("main".to_string()),
                    Instruction(
                        MPInstruction {
                            name: "li".to_string(),
                            arguments: vec![
                                Register(Normal(Named("t0".to_string()))),
                                Number(Immediate(I16(17))),
                            ],
                        }
                    ),
                    Instruction(
                        MPInstruction {
                            name: "li".to_string(),
                            arguments: vec![
                                Register(Normal(Named("t1".to_string()))),
                                Number(Immediate(I16(25))),
                            ],
                        }
                    ),
                    Instruction(
                        MPInstruction {
                            name: "add".to_string(),
                            arguments: vec![
                                Register(Normal(Named("t2".to_string()))),
                                Register(Normal(Named("t1".to_string()))),
                                Register(Normal(Named("t0".to_string()))),
                            ],
                        }
                    ),
                    Instruction(
                        MPInstruction {
                            name: "move".to_string(),
                            arguments: vec![
                                Register(Normal(Named("a0".to_string()))),
                                Register(Normal(Named("t2".to_string()))),
                            ],
                        }
                    ),
                    Instruction(
                        MPInstruction {
                            name: "li".to_string(),
                            arguments: vec![
                                Register(Normal(Named("v0".to_string()))),
                                Number(Immediate(I16(1))),
                            ],
                        }
                    ),
                    Instruction(
                        MPInstruction {
                            name: "syscall".to_string(),
                            arguments: vec![],
                        },
                    ),
                    Instruction(
                        MPInstruction {
                            name: "li".to_string(),
                            arguments: vec![
                                Register(Normal(Named("a0".to_string()))),
                                Number(Char('\n')),
                            ],
                        }
                    ),
                    Instruction(
                        MPInstruction {
                            name: "li".to_string(),
                            arguments: vec![
                                Register(Normal(Named("v0".to_string()))),
                                Number(Immediate(I16(11))),
                            ],
                        }
                    ),
                    Instruction(
                        MPInstruction {
                            name: "syscall".to_string(),
                            arguments: vec![],
                        }
                    ),
                    Instruction(
                        MPInstruction {
                            name: "li".to_string(),
                            arguments: vec![
                                Register(Normal(Named("v0".to_string()))),
                                Number(Immediate(I16(0))),
                            ],
                        }
                    ),
                    Instruction(
                        MPInstruction {
                            name: "jr".to_string(),
                            arguments: vec![
                                Register(Normal(Named("ra".to_string()))),
                            ],
                        }
                    ),
                ],
            }
        );
    }
}