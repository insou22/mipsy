use std::{ascii, str::FromStr};

use crate::interactive::{error::CommandError, prompt};

use super::*;
use colored::*;
use mipsy_lib::Register;
use mipsy_parser::*;

#[allow(clippy::format_in_format_args)]
pub(crate) fn print_command() -> Command {
    command(
        "print",
        vec!["p"],
        vec!["item"],
        vec!["format"],
        "print an item - a register, value in memory, etc.",
        |state, label, args| {
            if label == "_help" {
                return Ok(
                    format!(
                        "Prints the current value of an {0} in the loaded program.\n\
                         {0} can be one of:\n\
                    \x20- a {1}: named (`{2}{3}`) or numbered (`{2}{4}`),\n\
                    \x20- a {5} {1}: `{2}{6}`, `{2}{7}`, `{2}{8}`,\n\
                    \x20- an {9}: decimal (`4194304`), hex (`{10}400000`), labelled (`{11}`),\n\
                    \x20- {12}: `{2}{13}` - prints all currently initialised registers.\n\
                         {14} can optionally be specified (default: `{15}`) to specify how the value\n\
                    \x20 should be printed. Options: `{16}`, `{17}`, `{15}`, `{18}{16}`, `{18}{17}`,\n\
                    \x20                             `{18}{15}` / `{19}{18}`, `{20}`, `{21}`.",
                        "<item>".magenta(),
                        "register".yellow().bold(),
                        "$".yellow(),
                        "t3".bold(),
                        "12".bold(),
                        "special".yellow().bold(),
                        "pc".bold(),
                        "hi".bold(),
                        "lo".bold(),
                        "address".yellow().bold(),
                        "0x".yellow(),
                        "my_label".yellow().bold(),
                        "all registers".yellow().bold(),
                        "all".bold(),
                        "[format]".magenta(),
                        format!("{}{}", "w".yellow().bold(), "ord".bold()),
                        format!("{}{}", "b".yellow().bold(), "yte".bold()),
                        format!("{}{}", "h".yellow().bold(), "alf".bold()),
                        "x".yellow().bold(),
                        "he".bold(),
                        format!("{}{}", "c".yellow().bold(), "har".bold()),
                        format!("{}{}", "s".yellow().bold(), "tring".bold()),
                    ),
                )
            }

            let get_error = || CommandError::WithTip { 
                error: Box::new(CommandError::BadArgument { arg: "<item>".magenta().to_string(), instead: args[0].to_string() }),
                tip: format!("try `{}`", "help print".bold()),
            };

            let arg = mipsy_parser::parse_argument(&args[0], state.config.tab_size)
                    .map_err(|_| get_error())?;

            let print_type = &*args.get(1).cloned().unwrap_or_else(|| "word".to_string());
            match print_type {
                "byte" | "half" | "word" | "xbyte" | "xhalf" | "xword" | "hex" | "char" | "string" |
                "b"    | "h"    | "w"    | "xb"    | "xh"    | "xw"    |   "x" | "c"    | "s" => {}
                other => {
                    return Err(
                        CommandError::BadArgument { arg: "[format]".magenta().to_string(), instead: other.to_string() }
                    );
                }
            }

            let binary  = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
            let runtime = state.runtime.as_ref().ok_or(CommandError::MustLoadFile)?;

            match arg {
                MpArgument::Register(MpRegister::Normal(ident)) => {
                    match print_type {
                        "string" | "s" => {
                            prompt::error(format!("{} `string` unsupported for {} `register`", "[format]".magenta(), "<item>".magenta()));
                            prompt::tip_nl(format!("try using an address instead - `{}`", "help print".bold()));
                            return Ok("".into());
                        }
                        _ => {}
                    }

                    if matches!(ident, MpRegisterIdentifier::Named(ref name) if name == "all") {
                        for register in &Register::all() {
                            if let Ok(val) = runtime.timeline().state().read_register(register.to_u32()) {
                                let out = format_simple_print(val, print_type);
                                println!("{}{:4} = {}", "$".yellow(), register.to_lower_str().bold(), out);
                            }
                        }

                        if let Ok(val) = runtime.timeline().state().read_lo() {
                            println!(" {:4} = {}", "lo".bold(), format_simple_print(val, print_type));
                        }

                        if let Ok(val) = runtime.timeline().state().read_hi() {
                            println!(" {:4} = {}", "hi".bold(), format_simple_print(val, print_type));
                        }

                        println!(" {:4} = {}", "pc".bold(), format_simple_print(runtime.timeline().state().pc() as i32, print_type));
                    } else {
                        let (val, reg_name) = 
                        {
                            let (unchecked_val, reg_name) = match ident {
                                MpRegisterIdentifier::Named(name) => {
                                    let name = name.to_ascii_lowercase();

                                    if name == "pc" {
                                        Ok((Ok(runtime.timeline().state().pc() as i32), "pc"))
                                    } else if name == "hi" {
                                        Ok((runtime.timeline().state().read_hi(), "hi"))
                                    } else if name == "lo" {
                                        Ok((runtime.timeline().state().read_lo(), "lo"))
                                    } else {
                                        Register::from_str(&name)
                                            .map(|reg| (runtime.timeline().state().read_register(reg.to_u32()), reg.to_lower_str()))
                                            .map_err(|_| CommandError::UnknownRegister { register: name })
                                    }
                                },
                                MpRegisterIdentifier::Numbered(num) => {
                                    Register::from_number(num as i32)
                                        .map(|reg| (runtime.timeline().state().read_register(reg.to_u32()), reg.to_lower_str()))
                                        .map_err(|_| CommandError::UnknownRegister { register: num.to_string() })
                                }
                            }?;

                            let val = match unchecked_val {
                                Ok(val) => val,
                                Err(_)  => {
                                    prompt::error_nl(format!("{}{} is uninitialized", "$".yellow(), reg_name.bold()));
                                    return Ok("".into());
                                }
                            };

                            (val, reg_name)
                        };

                        let value = format_simple_print(val, print_type);
                        prompt::success_nl(format!("{}{} = {}", "$".yellow(), reg_name.bold(), value));
                    }
                }
                MpArgument::Number(MpNumber::Immediate(imm)) => {
                    let imm = match imm {
                        MpImmediate::I16(imm) => {
                            imm as u32
                        }
                        MpImmediate::U16(imm) => {
                            imm as u32
                        }
                        MpImmediate::I32(imm) => {
                            imm as u32
                        }
                        MpImmediate::U32(imm) => {
                            imm
                        }
                        MpImmediate::LabelReference(label) => {
                            binary.get_label(&label)
                                    .map_err(|_| CommandError::UnknownLabel { label: label.to_string() })?
                        }
                    };

                    let map_err = |_err| CommandError::UninitialisedPrint { addr: imm };

                    let value = match print_type {
                        "byte"  | "b"  => format!("{}", runtime.timeline().state().read_mem_byte(imm).map_err(map_err)?),
                        "half"  | "h"  => format!("{}", runtime.timeline().state().read_mem_half(imm).map_err(map_err)?),
                        "word"  | "w"  => format!("{}", runtime.timeline().state().read_mem_word(imm).map_err(map_err)?),
                        "xbyte" | "xb" => format!("0x{:02x}", runtime.timeline().state().read_mem_byte(imm).map_err(map_err)? as u8),
                        "xhalf" | "xh" => format!("0x{:04x}", runtime.timeline().state().read_mem_half(imm).map_err(map_err)? as u16),
                        "xword" | "xw" | "hex" | "x" => format!("0x{:08x}", runtime.timeline().state().read_mem_word(imm).map_err(map_err)? as u32),
                        "char"  | "c"  => format!("\'{}\'", ascii::escape_default(runtime.timeline().state().read_mem_byte(imm).map_err(map_err)? as u8)),
                        "string"| "s"  => {
                            let mut text = String::new();

                            let mut addr = imm;
                            loop {
                                let chr =
                                    match runtime.timeline().state().read_mem_byte(addr) {
                                        Ok(byte) => byte,
                                        Err(_) => {
                                            return Err(CommandError::UnterminatedString { good_parts: text });
                                        }   
                                    };

                                if chr == 0 {
                                    break;
                                }

                                text.push_str(&ascii::escape_default(chr).to_string());
                                addr += 1;
                            }

                            format!("\"{}\"", text)
                        },
                        _ => unreachable!(),
                    };

                    prompt::success_nl(format!("{} = {}", args[0], value));
                }
                _ => return Err(get_error()),
            }

            Ok("".into())
        }
    )
}

fn format_simple_print(val: i32, print_type: &str) -> String {
    match print_type {
        "byte"  | "b"  => format!("{}", val & 0xFF),
        "half"  | "h"  => format!("{}", val & 0xFFFF),
        "word"  | "w"  => format!("{}", val),
        "xbyte" | "xb" => format!("0x{:02x}", (val as u32) & 0xFF),
        "xhalf" | "xh" => format!("0x{:04x}", (val as u32) & 0xFFFF),
        "xword" | "xw" | "hex" | "x" => format!("0x{:08x}", val as u32),
        "char"  | "c"  => format!("\'{}\'", ascii::escape_default((val & 0xFF) as u8)),
        _ => unreachable!(),
    }
}
