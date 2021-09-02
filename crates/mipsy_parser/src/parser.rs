use std::rc::Rc;

use crate::{ErrorLocation, Span, attribute::{Attribute, parse_inner_attribute, parse_outer_attribute}, directive::{
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

pub struct TaggedFile<'tag, 'file> {
    tag: Option<&'tag str>,
    file_contents: &'file str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MpProgram {
    pub(crate) items: Vec<MpAttributedItem>,
    pub(crate) file_attributes: Vec<Attribute>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MpAttributedItem {
    pub(crate) item: MpItem,
    pub(crate) attributes: Vec<Attribute>,
    pub(crate) file_tag: Option<Rc<str>>,
    pub(crate) line_number: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MpItem {
    Instruction(MpInstruction),
    Directive(MpDirective),
    Label(MpLabel),
}

impl<'tag, 'file> TaggedFile<'tag, 'file> {
    pub fn new(tag: Option<&'tag str>, file_contents: &'file str) -> Self {
        Self {
            tag,
            file_contents,
        }
    }
}

impl MpAttributedItem {
    pub fn new(item: MpItem, attributes: Vec<Attribute>, file_tag: Option<Rc<str>>, line_number: u32) -> Self {
        Self {
            item,
            attributes,
            file_tag,
            line_number,
        }
    }

    pub fn item(&self) -> &MpItem {
        &self.item
    }

    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    pub fn file_tag(&self) -> Option<Rc<str>> {
        self.file_tag.clone()
    }

    pub fn line_number(&self) -> u32 {
        self.line_number
    }
}

impl MpProgram {
    pub fn new(items: Vec<MpAttributedItem>, file_attributes: Vec<Attribute>) -> Self {
        Self {
            items,
            file_attributes,
        }
    }

    pub fn items(&self) -> &[MpAttributedItem] {
        &self.items
    }

    pub fn items_mut(&mut self) -> &mut Vec<MpAttributedItem> {
        &mut self.items
    }

    fn merge(&mut self, mut other: MpProgram) {
        if !self.items.is_empty() {
            self.items.push(MpAttributedItem {
                item: MpItem::Directive(MpDirective::Text),
                attributes: vec![],
                file_tag: None,
                line_number: 0,
            });
        }

        self.items.append(&mut other.items);
    }
}

pub fn parse_mips_item(i: Span<'_>) -> IResult<Span<'_>, (MpItem, Vec<Attribute>, u32)> {
    map(
        tuple((
            comment_multispace0,
            many0(
                map(
                    tuple((
                        parse_inner_attribute,
                        comment_multispace0,
                    )),
                    |(attr, _)| attr,
                )
            ),
            comment_multispace0,
            position,
            alt((
                map(parse_label,       MpItem::Label),
                map(parse_directive,   MpItem::Directive),
                map(parse_instruction, MpItem::Instruction),
            )),
            comment_multispace0,
        )),
        |(_, attrs,  _, pos, item, _)| (item, attrs, pos.location_line())
    )(i)
}


pub fn parse_mips_bytes<'a>(file_name: Option<Rc<str>>) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, MpProgram> {
    move |i| {
        let (
            remaining_input,
            (attrs, items)
        ) = tuple((
            parse_outer_attributes,
            many0(
                alt((
                    map(
                        parse_mips_item,
                        |(item, attrs, line)| Some(MpAttributedItem {
                            item,
                            attributes: attrs,
                            file_tag: file_name.clone(),
                            line_number: line,
                        }),
                    ),
                    map(comment_multispace1, |_| None),
                ))
            )
        ))(i)?;

        let items = items.into_iter()
            .flatten()
            .collect();

        Ok((
            remaining_input,
            MpProgram {
                items,
                file_attributes: attrs,
            },
        ))
    }
}

pub fn parse_outer_attributes(i: Span<'_>) -> IResult<Span<'_>, Vec<Attribute>> {
    map(
        tuple((
            comment_multispace0,
            many0(
                map(
                    tuple((
                        parse_outer_attribute,
                        comment_multispace0,
                    )),
                    |(attr, _)| attr,
                )
            ),
        )),
        |(_, attrs)| attrs
    )(i)
}

pub fn parse_mips(files: Vec<TaggedFile<'_, '_>>, default_tab_size: u32) -> Result<MpProgram, ErrorLocation> {
    let mut program = MpProgram {
        items: vec![],
        file_attributes: vec![],
    };

    for file in files {
        let file_name = file.tag;
        let input = file.file_contents;

        let file_name = file_name.map(Rc::from);

        let initial_file_string = crate::misc::tabs_to_spaces(input, default_tab_size);
        let initial_span = Span::new(initial_file_string.as_bytes());

        let (_remaining_input, outer_attrs) = parse_outer_attributes(initial_span)
            .expect("Initial outer attributes parser should never fail");

        let mut actual_tabsize = default_tab_size;

        for attr in outer_attrs {
            // TODO(zkol): Not a fan of this random hardcoding here
            if attr.key().to_ascii_lowercase() == "tabsize" {
                // TODO(zkol): This error handling needs to get wrapped up
                // with the rest somehow...
                actual_tabsize = attr.value().expect("Tabsize attribute requires a value")
                    .parse().expect("Tabsize attribute value should be numeric");
            }
        }

        let file_string = crate::misc::tabs_to_spaces(input, actual_tabsize);
        let span = Span::new(file_string.as_bytes());

        let result = parse_result(span, file_name.clone(), parse_mips_bytes(file_name))?;

        program.merge(result);
    }

    Ok(program)
}