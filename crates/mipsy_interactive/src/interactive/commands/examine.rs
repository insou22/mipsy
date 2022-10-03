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
        |_, state, label, _args| {
            if label == "__help__" {
                return Ok(
                    "TODO: long form help".into()
                )
            }

            let pages = &state.binary.as_ref().ok_or(CommandError::MustLoadFile)?.data;

            const ROW_SIZE: usize = 16;
            let rows: usize = pages.len() / ROW_SIZE + if pages.len() == 0 {0} else {1};
            let offset: usize = ROW_SIZE * 5 / 2;

            for nth in 0..rows {
                let mut byte_repr = String::with_capacity(ROW_SIZE * 2);
                let mut printable_repr = String::with_capacity(ROW_SIZE);

                for (i, offset) in (0..ROW_SIZE).enumerate() {
                    // print in groups of 2 (`xxd` format)
                    if i % 2 == 0 { byte_repr.push(' '); }

                    let byte = pages.get(nth * ROW_SIZE + offset);
                    if let Some(&byte) = byte {
                        byte_repr.push_str(render_data(byte).as_ref());
                        printable_repr.push_str(byte.as_option()
                            .map(|&value| value as u32)
                            .and_then(char::from_u32)
                            .map(|c| c.escape())
                            .unwrap_or("_".bright_black().to_string())
                            .as_ref()
                        );
                    } else {
                        // end of allocated memory
                        break;
                    }
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
