use colored::Colorize;
use mipsy_lib::{DATA_BOT, Safe};
use mipsy_parser::{MpArgument, MpImmediate, MpNumber};

use crate::interactive::error::CommandError;

use super::*;

pub(crate) fn examine_command() -> Command {
    command(
        "examine",
        vec!["e", "ex", "x", "dump"],
        vec![],
        vec![],
        vec![],
        "examine memory contents",
        |_, state, label, mut args| {
            // TODO:
            // - long help
            // - ability to select section being printed
            // - <enter> to examine the next chunk of memory
            if label == "__help__" {
                return Ok(
                    "TODO: long form help".into()
                )
            }

            let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
            let mut pages = &binary.data;
            let section_base = DATA_BOT as usize;
            let mut base_addr = DATA_BOT as usize;
            let mut dump_len = pages.len().min(256);

            if let Some(len) = args.get(0).and_then(|num| num.parse::<usize>().ok()) {
                args = &args[1..];
                // allow the default limit to be overridden but not past the end of data segment
                dump_len = pages.len().min(len);
            }

            // TODO: make this work for labels above the region
            // TODO: more informative error
            if let Some(base) = args.get(0).map(|arg| parse_arg(state, arg)) {
                base_addr = base? as usize;
            }
            let base_diff = base_addr.checked_sub(section_base).ok_or(CommandError::AddressNotInSection)?;

            let not_inlined = pages[base_diff..].to_vec();
            pages = &not_inlined;
            dump_len = dump_len.min(pages.len());

            let default_size = 16;
            let row_size: usize = termsize::get().map_or(default_size, |size|
                // subtract "0x{:8x}: " length and allow for extra length in representation
                // TODO: allow longer than 16? can be done by removing .min() but not very readable
                (((size.cols - 12 - 1) * 2 / 7) as usize).min(default_size)
            ).max(1);

            let mut rows: usize = dump_len / row_size;
            if dump_len % row_size != 0 { rows += 1; }
            let offset: usize = row_size * 5 / 2;

            for nth in 0..rows {
                let mut label_repr = String::with_capacity(row_size * 3);
                let mut byte_repr  = String::with_capacity(row_size * 3);
                let mut printable_repr = String::with_capacity(row_size);

                for offset in 0..row_size {
                    // print in groups of 2 (`xxd` format)
                    if offset % 2 == 0 { byte_repr.push(' '); }

                    // reached end of dump and/or allocated memory
                    let index = nth * row_size + offset;
                    if index >= dump_len { break };

                    let address = base_addr + index;
                    let byte = state.runtime.timeline().state()
                        .read_mem_byte_uninit(address as u32)
                        .unwrap();

                    if let Some((label, _)) = binary.labels.iter().find(|(_, &addr)| addr == address as u32) {
                        if let Some(padding) = byte_repr.len().checked_sub(label_repr.len()) {
                            label_repr.push_str(" ".repeat(padding).as_ref());
                        } else {
                            // labels overlap - truncate previous label
                            label_repr.truncate(byte_repr.len() - 1);
                            label_repr.push(' ');
                        }
                        label_repr.push_str(label);
                    }

                    byte_repr.push_str(render_data(byte).as_ref());
                    printable_repr.push_str(byte.as_option()
                        .map(|&value| value as u32)
                        .and_then(char::from_u32)
                        .map(|c| c.escape())
                        .unwrap_or("_".bright_black().to_string())
                        .as_ref()
                    );
                }

                if !label_repr.is_empty() {
                    println!("{} {}", " ".repeat(10), label_repr.yellow().bold());
                }
                println!("{}{:08x}:{:offset$}  {}",
                    "0x".yellow(), base_addr as usize + nth * row_size,
                    byte_repr,
                    printable_repr,
                );
            }

            Ok("".into())
        }
    )
}

fn render_data(data_val: Safe<u8>) -> String {
    match data_val {
        Safe::Valid(byte) => {
            format!("{:02x}", byte)
        }
        Safe::Uninitialised => {
            format!("__")
        }
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
    if !command_name.is_empty() { help.push(' ') };

    CommandError::WithTip {
        error: Box::new(error),
        tip: format!("try `{}{}`", help.bold(), command_name.bold()),
    } 
}

fn parse_arg(state: &State, arg: &String) -> Result<u32, CommandError> {
    let get_error = |expected: &str| generate_err(
        CommandError::BadArgument { arg: expected.magenta().to_string(), instead: arg.into() },
        &String::from(""),
    );

    let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
    let arg = mipsy_parser::parse_argument(arg, state.config.tab_size)
            .map_err(|_| get_error("<addr>"))?;

    if let MpArgument::Number(MpNumber::Immediate(ref imm)) = arg {
        Ok(match imm {
            MpImmediate::I16(imm) => *imm as u32,
            MpImmediate::U16(imm) => *imm as u32,
            MpImmediate::I32(imm) => *imm as u32,
            MpImmediate::U32(imm) => *imm,
            MpImmediate::LabelReference(label) =>
                binary.get_label(label)
                    .map_err(|_| CommandError::UnknownLabel { label: label.to_string() })?,
        })
    } else {
        Err(get_error("<addr>"))
    }
}
