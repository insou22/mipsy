use std::rc::Rc;

use crate::{ErrorLocation, Span, directive::{
        MpDirective,
        parse_directive,
    }, instruction::{
        MpInstruction,
        parse_instruction,
    }, label::{MpLabel, parse_label}, misc::{comment_multispace0, comment_multispace1, parse_result}};
use nom::{
    IResult,
    sequence::tuple,
    combinator::map,
    multi::many0,
    branch::alt,
};
use nom_locate::position;


#[derive(Debug, Clone, PartialEq)]
pub struct MpProgram {
    pub(crate) items: Vec<(MpItem, Option<Rc<str>>, u32)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MpItem {
    Instruction(MpInstruction),
    Directive(MpDirective),
    Label(MpLabel),
}

impl MpProgram {
    pub fn new(items: Vec<(MpItem, Option<Rc<str>>, u32)>) -> Self {
        Self {
            items,
        }
    }

    pub fn items(&self) -> &[(MpItem, Option<Rc<str>>, u32)] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Vec<(MpItem, Option<Rc<str>>, u32)> {
        &mut self.items
    }

    fn merge(&mut self, mut other: MpProgram) {
        if !self.items.is_empty() {
            self.items.push((MpItem::Directive(MpDirective::Text), None, 0));
        }

        self.items.append(&mut other.items);
    }
}

pub fn parse_mips_item(i: Span<'_>) -> IResult<Span<'_>, (MpItem, u32)> {
    map(
        tuple((
            comment_multispace0,
            position,
            alt((
                map(parse_label,       MpItem::Label),
                map(parse_directive,   MpItem::Directive),
                map(parse_instruction, MpItem::Instruction),
            )),
            comment_multispace0,
        )),
        |(_, pos, item, _)| (item, pos.location_line())
    )(i)
}


pub fn parse_mips_bytes<'a>(file_name: Option<Rc<str>>) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, MpProgram> {
    move |i| {
        let (
            remaining_input,
            items
        ) = many0(
            alt((
                map(
                    parse_mips_item,
                    |(item, line)| Some((item, file_name.clone(), line)),
                ),
                map(comment_multispace1, |_| None),
            ))
        )(i)?;

        let items = items.into_iter()
            .filter_map(|x| x)
            .collect();

        Ok((
            remaining_input,
            MpProgram {
                items
            },
        ))
    }
}

pub fn parse_mips(files: Vec<(Option<&str>, &str)>) -> Result<MpProgram, ErrorLocation> {
    let mut program = MpProgram {
        items: vec![],    
    };

    for file in files {
        let (file_name, input) = file;

        let file_name = file_name.map(Rc::from);

        let string = crate::misc::tabs_to_spaces(input);
        let result = parse_result(Span::new(string.as_bytes()), file_name.clone(), parse_mips_bytes(file_name))?;

        program.merge(result);
    }

    Ok(program)
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::MPItem::*;
//     use crate::MPArgument::*;
//     use crate::MPRegister::*;
//     use crate::MPRegisterIdentifier::*;
//     use crate::MPNumber::*;
//     use crate::MPImmediate::*;

//     #[cfg(test)]
//     const ADD_S: &str = " # add 17 and 25  and print result

// main:                    #  x, y, z in $t0, $t1, $t2,
//     li   $t0, 17         # x = 17;

//     li   $t1, 25         # y = 25;

//     add  $t2, $t1, $t0   # z = x + y

//     move $a0, $t2        # printf(\"%d\", a0);
//     li   $v0, 1
//     syscall

//     li   $a0, '\\n'       # printf(\"%c\", '\\n');
//     li   $v0, 11
//     syscall

//     li   $v0, 0          # return 0
//     jr   $ra
// ";

//     #[test]
//     fn add_s() {
//         assert_eq!(
//             parse_mips(ADD_S).unwrap(),
//             MPProgram {
//                 items: vec![
//                     (Label("main".to_string()), 3),
//                     (Instruction(
//                         MPInstruction {
//                             name: "li".to_string(),
//                             arguments: vec![
//                                 (Register(Normal(Named("t0".to_string()))), 10),
//                                 (Number(Immediate(I16(17))), 15),
//                             ],
//                             col: 5,
//                         }
//                     ), 4),
//                     (Instruction(
//                         MPInstruction {
//                             name: "li".to_string(),
//                             arguments: vec![
//                                 (Register(Normal(Named("t1".to_string()))), 10),
//                                 (Number(Immediate(I16(25))), 15),
//                             ],
//                             col: 5,
//                         }
//                     ), 6),
//                     (Instruction(
//                         MPInstruction {
//                             name: "add".to_string(),
//                             arguments: vec![
//                                 (Register(Normal(Named("t2".to_string()))), 10),
//                                 (Register(Normal(Named("t1".to_string()))), 15),
//                                 (Register(Normal(Named("t0".to_string()))), 20),
//                             ],
//                             col: 5,
//                         }
//                     ), 8),
//                     (Instruction(
//                         MPInstruction {
//                             name: "move".to_string(),
//                             arguments: vec![
//                                 (Register(Normal(Named("a0".to_string()))), 10),
//                                 (Register(Normal(Named("t2".to_string()))), 15),
//                             ],
//                             col: 5,
//                         }
//                     ), 10),
//                     (Instruction(
//                         MPInstruction {
//                             name: "li".to_string(),
//                             arguments: vec![
//                                 (Register(Normal(Named("v0".to_string()))), 10),
//                                 (Number(Immediate(I16(1))), 15),
//                             ],
//                             col: 5,
//                         }
//                     ), 11),
//                     (Instruction(
//                         MPInstruction {
//                             name: "syscall".to_string(),
//                             arguments: vec![],
//                             col: 5,
//                         },
//                     ), 12),
//                     (Instruction(
//                         MPInstruction {
//                             name: "li".to_string(),
//                             arguments: vec![
//                                 (Register(Normal(Named("a0".to_string()))), 10),
//                                 (Number(Char('\n')), 15),
//                             ],
//                             col: 5,
//                         }
//                     ), 14),
//                     (Instruction(
//                         MPInstruction {
//                             name: "li".to_string(),
//                             arguments: vec![
//                                 (Register(Normal(Named("v0".to_string()))), 10),
//                                 (Number(Immediate(I16(11))), 15),
//                             ],
//                             col: 5,
//                         }
//                     ), 15),
//                     (Instruction(
//                         MPInstruction {
//                             name: "syscall".to_string(),
//                             arguments: vec![],
//                             col: 5,
//                         }
//                     ), 16),
//                     (Instruction(
//                         MPInstruction {
//                             name: "li".to_string(),
//                             arguments: vec![
//                                 (Register(Normal(Named("v0".to_string()))), 10),
//                                 (Number(Immediate(I16(0))), 15),
//                             ],
//                             col: 5,
//                         }
//                     ), 18),
//                     (Instruction(
//                         MPInstruction {
//                             name: "jr".to_string(),
//                             arguments: vec![
//                                 (Register(Normal(Named("ra".to_string()))), 10),
//                             ],
//                             col: 5,
//                         }
//                     ), 19),
//                 ],
//             }
//         );
//     }
// }