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
            if label == "__help__" {
                // TODO:
                // - long help
                // - include labels where relevant?
                // - ability to select section being printed
                return Ok(
                    "TODO: long form help".into()
                )
            }

            let binary = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
            let mut pages = &binary.data;
            let section_base = DATA_BOT as usize;
            let mut base_addr = DATA_BOT as usize;
            let mut dump_len = pages.len();

            if let Some(len) = args.get(0).and_then(|num| num.parse::<usize>().ok()) {
                args = &args[1..];
                dump_len = dump_len.min(len);
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

            const ROW_SIZE: usize = 16;
            let mut rows: usize = dump_len / ROW_SIZE;
            if dump_len % ROW_SIZE != 0 { rows += 1; }
            let offset: usize = ROW_SIZE * 5 / 2;

            for nth in 0..rows {
                let mut byte_repr = String::with_capacity(ROW_SIZE * 2);
                let mut printable_repr = String::with_capacity(ROW_SIZE);

                for (i, offset) in (0..ROW_SIZE).enumerate() {
                    // print in groups of 2 (`xxd` format)
                    if i % 2 == 0 { byte_repr.push(' '); }

                    // reached end of dump and/or allocated memory
                    let index = nth * ROW_SIZE + offset;
                    if index >= dump_len { break };

                    let byte = pages[index];
                    byte_repr.push_str(render_data(byte).as_ref());
                    printable_repr.push_str(byte.as_option()
                        .map(|&value| value as u32)
                        .and_then(char::from_u32)
                        .map(|c| c.escape())
                        .unwrap_or("_".bright_black().to_string())
                        .as_ref()
                    );
                }

                println!("{}{:08x}:{:offset$}  {}",
                    "0x".yellow(), base_addr as usize + nth * ROW_SIZE,
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

    if arg.contains(':') {
        // parts contains at least 2 elements
        let mut parts = arg.split(':');
        let mut file = parts.next().unwrap();
        if file.is_empty() {
            let mut filenames = binary.line_numbers.values()
                    .map(|(filename, _)| filename);
            file = filenames.next().unwrap();
            if !filenames.all(|f| f.as_ref() == file) {
                return Err(CommandError::MustSpecifyFile);
            }
        }

        let line_number: u32 = parts.next().unwrap().parse().map_err(|_| get_error("<line number>"))?;
        let mut lines = binary.line_numbers.iter()
            .filter(|(_, (filename, _))| filename.as_ref() == file).collect::<Vec<_>>();
        lines.sort_unstable_by(|a, b| a.1.1.cmp(&b.1.1));

        // use first line after the specified line that contains an instruction
        let addr = lines.iter()
            .find(|(_, &(_, _line_number))| _line_number >= line_number)
            .ok_or_else(|| CommandError::LineDoesNotExist { line_number })?.0;

        return Ok(*addr)
    }

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
