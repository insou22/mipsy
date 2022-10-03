use colored::Colorize;
use mipsy_lib::{DATA_BOT, Safe};

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
                // - ability to select exact start of dump e.g. x 20 my_label
                return Ok(
                    "TODO: long form help".into()
                )
            }

            let pages = &state.binary.as_ref().ok_or(CommandError::MustLoadFile)?.data;
            let mut dump_len = pages.len();

            if let Some(len) = args.get(0).and_then(|num| num.parse::<usize>().ok()) {
                args = &args[1..];
                dump_len = dump_len.min(len);
            }

            const ROW_SIZE: usize = 16;
            let rows: usize = dump_len / ROW_SIZE + if dump_len == 0 {0} else {1};
            let offset: usize = ROW_SIZE * 5 / 2;

            for nth in 0..rows {
                let mut byte_repr = String::with_capacity(ROW_SIZE * 2);
                let mut printable_repr = String::with_capacity(ROW_SIZE);

                for (i, offset) in (0..ROW_SIZE).enumerate() {
                    // print in groups of 2 (`xxd` format)
                    if i % 2 == 0 { byte_repr.push(' '); }

                    // reached end of dump and/or allocated memory
                    if nth * ROW_SIZE + offset >= dump_len { break };

                    let byte = pages[nth * ROW_SIZE + offset];
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
                    "0x".yellow(), DATA_BOT as usize + nth * ROW_SIZE,
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
