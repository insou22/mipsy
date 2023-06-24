use colored::Colorize;
use mipsy_lib::{
    util::{get_segment, Segment},
    Register, Safe,
};
use mipsy_parser::{MpArgument, MpImmediate, MpNumber};
use std::{fmt::Display, str::FromStr};

use crate::interactive::error::CommandError;

use super::*;

pub(crate) fn examine_command() -> Command {
    command(
        "examine",
        vec!["e", "ex", "x", "dump"],
        vec![],
        vec!["section", "len", "addr", "-nolabels"],
        vec![],
        "examine memory contents",
        |_, state, label, mut args| {
            // TODO: <enter> to examine the next chunk of memory
            if label == "__help__" {
                return Ok(
                    format!(
                        "Examine memory contents in a format akin to the tool `xxd`.\n\
                         {0} may be: `.data` (default), `.text`, `.stack`, `.kdata`, `.ktext`.\n\
                         {1} controls the maximum number of bytes displayed.\n\
                         {2} controls where the memory dump starts (by default the start of the section).\n\
                         {2} may be: a register name (`$t0`, `t0`), a register number (`$14`, 14),\n\
                    \x20             a decimal address (`4194304`), a hex address (`{3}400000`),\n\
                    \x20             or a label (`{4}`).\n\
                         If {7} is provided, then label names will be included in the output.\n\
                         Unprintable bytes are displayed as {5}, and uninitialized bytes are displayed as {6}.\n\
                        ",
                        "<section>".magenta(),
                        "<length>".magenta(),
                        "<addr>".magenta(),
                        "0x".yellow(),
                        "main".yellow().bold(),
                        ".".bright_black(),
                        "_".bright_black(),
                        "-nolabels".magenta(),
                    )
                );
            }

            let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;

            let mut segment = if let Some(segment) =
                args.get(0).and_then(|segment| match segment.as_ref() {
                    ".data" => Some(Segment::Data),
                    ".text" => Some(Segment::Text),
                    ".stack" => Some(Segment::Stack),
                    ".kdata" => Some(Segment::KData),
                    ".ktext" => Some(Segment::KText),
                    _ => None,
                }) {
                args = &args[1..];
                segment
            } else {
                Segment::Data
            };

            let dump_len = if let Some(len) = args.get(0).and_then(|num| num.parse::<usize>().ok())
            {
                args = &args[1..];
                len
            } else {
                128
            };

            let mut base_addr = if let Some(base) = args.get(0).map(|arg| parse_arg(state, arg)) {
                if base.is_ok() {
                    args = &args[1..];
                }
                base
            } else {
                Ok(segment.get_lower_bound())
            };

            let hide_labels = args
                .get(0)
                .map_or(false, |a| a == &String::from("-nolabels"));
            if hide_labels && base_addr.is_err() {
                // if -labels was provided, ensure base_addr is valid
                base_addr = Ok(segment.get_lower_bound());
            }

            let base_addr = base_addr? as usize;

            // for most cases this is a no-op, but if given an address in the stack we should
            // reverse the order of the scan
            segment = get_segment(base_addr as u32);

            let default_size = 16;
            let row_size: usize = termsize::get()
                .map_or(default_size, |size|
                // subtract "0x{:8x}: " length and allow for extra length in representation
                // TODO: allow rows longer than 16 bytes? can be done by removing .min() but not very readable
                (((size.cols - 12 - 1) * 2 / 7) as usize).min(default_size))
                .max(1);

            let mut rows: usize = dump_len / row_size;
            if dump_len % row_size != 0 {
                rows += 1;
            }
            let offset: usize = row_size * 5 / 2;

            for nth in 0..rows {
                let mut label_strs: Vec<WrappedString> = Vec::new();
                let mut arrow_tips = WrappedString::new();
                let mut byte_repr = String::with_capacity(row_size * 3);
                let mut printable_repr = String::with_capacity(row_size);

                for offset in 0..row_size {
                    // print in groups of 2 (`xxd` format)
                    if offset % 2 == 0 {
                        byte_repr.push(' ');
                    }

                    // reached end of dump and/or allocated memory
                    let index = nth * row_size + offset;
                    if index >= dump_len {
                        break;
                    };

                    // automatically move upwards when displaying stack
                    let address = if segment == Segment::Stack {
                        base_addr - index
                    } else {
                        base_addr + index
                    };

                    let byte = state
                        .runtime
                        .timeline()
                        .state()
                        .read_mem_byte_uninit_unchecked(address as u32)
                        .unwrap();

                    // TODO: combine these when if-let chaining is stabilised
                    if !hide_labels {
                        if let Some((label, addr)) = binary
                            .labels
                            .iter()
                            .find(|(_, &addr)| addr == address as u32)
                        {
                            let offset = byte_repr.len();
                            label_strs.push(arrow_tips.clone());
                            label_strs.last_mut().unwrap().pad_insert(
                                offset,
                                format!(
                                    "{}: {}{}",
                                    label.bold().yellow(),
                                    "0x".yellow(),
                                    format!("{addr:08x}").purple()
                                )
                                .as_ref(),
                            );
                            arrow_tips.pad_insert(offset, "|");
                        }
                    }

                    byte_repr.push_str(render_data(byte).as_ref());
                    printable_repr.push_str(
                        byte.as_option()
                            .map(|&value| value as u32)
                            .and_then(char::from_u32)
                            .map(|c| c.escape())
                            .unwrap_or("_".bright_black().to_string())
                            .as_ref(),
                    );
                }

                let marker = if segment == Segment::Stack {
                    base_addr - nth * row_size
                } else {
                    base_addr + nth * row_size
                };

                if !label_strs.is_empty() {
                    label_strs
                        .iter()
                        .for_each(|l| println!("{} {}", " ".repeat(10), l));

                    println!("{} {}", " ".repeat(10), arrow_tips);
                }

                println!(
                    "{}{:08x}:{:offset$}  {}",
                    "0x".yellow(),
                    marker,
                    byte_repr,
                    printable_repr,
                );
            }

            Ok("".into())
        },
    )
}

fn render_data(data_val: Safe<u8>) -> String {
    match data_val {
        Safe::Valid(byte) => {
            format!("{:02x}", byte)
        }
        Safe::Uninitialised => "__".to_string(),
    }
}

#[derive(Clone)]
struct WrappedString(String);
impl WrappedString {
    fn new() -> Self {
        Self(String::new())
    }

    fn pad_insert(&mut self, offset: usize, string: &str) {
        self.0.push_str(" ".repeat(offset - self.0.len()).as_ref());
        self.0.insert_str(offset, string)
    }
}

impl Display for WrappedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

trait Escape {
    fn escape(&self) -> String;
}

impl Escape for char {
    fn escape(self: &char) -> String {
        match self {
            '\x20'..='\x7E' => self.to_string(), // printable ASCII
            _ => ".".bright_black().to_string(), // everything else
        }
    }
}

fn generate_err(error: CommandError, command_name: impl Into<String>) -> CommandError {
    let mut help = String::from("help breakpoint");
    let command_name = command_name.into();
    if !command_name.is_empty() {
        help.push(' ')
    };

    CommandError::WithTip {
        error: Box::new(error),
        tip: format!("try `{}{}`", help.bold(), command_name.bold()),
    }
}

fn parse_arg(state: &State, arg: &String) -> Result<u32, CommandError> {
    let get_error = |expected: &str| {
        generate_err(
            CommandError::BadArgument {
                arg: expected.magenta().to_string(),
                instead: arg.into(),
            },
            String::from(""),
        )
    };

    if let Ok(register) = Register::from_str(arg.strip_prefix('$').unwrap_or(arg)) {
        return Ok(state
            .runtime
            .timeline()
            .state()
            .read_register(register.to_u32())
            .map_err(|_| CommandError::UninitialisedRegister { register })?
            as u32);
    }

    let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
    let arg = mipsy_parser::parse_argument(arg, state.config.tab_size)
        .map_err(|_| get_error("<addr>"))?;

    if let MpArgument::Number(MpNumber::Immediate(ref imm)) = arg {
        Ok(match imm {
            MpImmediate::I16(imm) => *imm as u32,
            MpImmediate::U16(imm) => *imm as u32,
            MpImmediate::I32(imm) => *imm as u32,
            MpImmediate::U32(imm) => *imm,
            MpImmediate::LabelReference(label) => {
                binary
                    .get_label(label)
                    .map_err(|_| CommandError::UnknownLabel {
                        label: label.to_string(),
                    })?
            }
        })
    } else {
        Err(get_error("<addr>"))
    }
}
