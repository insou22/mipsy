use std::ascii;

use crate::interactive::{error::CommandError, prompt};

use super::*;
use colored::*;
use mipsy_lib::Register;
use mipsy_parser::*;

pub(crate) fn print_command() -> Command {
    command(
        "print",
        vec!["p"],
        vec!["item"],
        vec!["type"],
        "print an item - a register, value in memory, etc.",
        &format!(
            "Prints the current value of an {0} in the loaded program.\n\
             {0} can be one of:\n\
        \x20- a {13}: named (`{1}{2}`) or numbered (`{1}{3}`),\n\
        \x20- a {21} {13}: `{1}{18}`, `{1}{19}`, `{1}{20}`,\n\
        \x20- an {14}: decimal (`4194304`), hex (`{4}400000`), labelled (`{5}`),\n\
        \x20- {15}: `{1}{6}`.\n\
             {7} can optionally be specified (default: `{8}`) to specify how the value\n\
        \x20 should be printed. Options: `{9}`, `{10}`, `{8}`, `{16}{9}`, `{16}{10}`,\n\
        \x20                             `{16}{8}` or `{17}`, `{11}`, `{12}`.",
            "<item>".magenta(),
            "$".yellow(),
            "t3".bold(),
            "12".bold(),
            "0x".yellow(),
            "my_label".yellow().bold(),
            "all".bold(),
            "[type]".magenta(),
            "word".bold(),
            "byte".bold(),
            "half".bold(),
            "char".bold(),
            "string".bold(),
            "register".yellow().bold(),
            "address".yellow().bold(),
            "all registers".yellow().bold(),
            "x".bold(),
            "hex".bold(),
            "pc".bold(),
            "hi".bold(),
            "lo".bold(),
            "special".yellow().bold(),
        ),
        |state, _label, args| {
            let get_error = || CommandError::WithTip { 
                error: Box::new(CommandError::BadArgument { arg: "<item>".magenta().to_string(), instead: args[0].to_string() }),
                tip: format!("try `{}`", "help print".bold()),
            };

            let (leftover, arg) = mipsy_parser::parse_argument(args[0].as_bytes())
                    .map_err(|_| get_error())?;

            if !leftover.is_empty() {
                return Err(get_error());
            }

            let print_type = &*args.get(1).cloned().unwrap_or("word".to_string());
            match print_type {
                "byte" | "half" | "word" | "xbyte" | "xhalf" | "xword" | "hex" | "char" | "string" => {}
                other => {
                    return Err(
                        CommandError::BadArgument { arg: "<type>".magenta().to_string(), instead: other.to_string() }
                    );
                }
            }

            let binary  = state.binary.as_ref().ok_or(CommandError::MustLoadFile)?;
            let runtime = state.runtime.as_ref().ok_or(CommandError::MustLoadFile)?;

            match arg {
                MPArgument::Register(MPRegister::Normal(ident)) => {
                    if matches!(ident, MPRegisterIdentifier::Named(ref name) if name == "all") {
                        for register in &Register::all() {
                            match runtime.state().get_reg(register.to_u32()) {
                                Ok(val) => {
                                    let out = match print_type {
                                        "byte" | "half" | "word" | "xbyte" | "xhalf" | "xword" | "hex" | "char" => format_simple_print(val, print_type),
                                        "string" => {
                                            prompt::error(format!("{} `string` unsupported for {} `register`", "[type]".magenta(), "<item>".magenta()));
                                            prompt::tip_nl(format!("try using an address instead - `{}`", "help print".bold()));
                                            return Ok(());
                                        },
                                        _ => unreachable!(),
                                    };

                                    println!("{}{:4} = {}", "$".yellow(), register.to_lower_str().bold(), out);
                                }
                                Err(_) => {}
                            }
                        }
                        println!();
                    } else {
                        let (val, reg_name) = 
                        {
                            let (unchecked_val, reg_name) = match ident {
                                MPRegisterIdentifier::Named(name) => {
                                    let name = name.to_ascii_lowercase();

                                    if name == "pc" {
                                        Ok((Ok(runtime.state().get_pc() as i32), "pc"))
                                    } else if name == "hi" {
                                        Ok((runtime.state().get_hi(), "hi"))
                                    } else if name == "lo" {
                                        Ok((runtime.state().get_lo(), "lo"))
                                    } else {
                                        Register::from_str(&name)
                                            .map(|reg| (runtime.state().get_reg(reg.to_u32()), reg.to_lower_str()))
                                            .map_err(|_| CommandError::UnknownRegister { register: name })
                                    }
                                },
                                MPRegisterIdentifier::Numbered(num) => {
                                    Register::from_number(num as i32)
                                        .map(|reg| (runtime.state().get_reg(reg.to_u32()), reg.to_lower_str()))
                                        .map_err(|_| CommandError::UnknownRegister { register: num.to_string() })
                                }
                            }?;

                            let val = match unchecked_val {
                                Ok(val) => val,
                                Err(_)  => {
                                    prompt::error_nl(format!("{}{} is uninitialized", "$".yellow(), reg_name.bold()));
                                    return Ok(());
                                }
                            };

                            (val, reg_name)
                        };

                        let value = match print_type {
                            "byte" | "half" | "word" | "xbyte" | "xhalf" | "xword" | "hex" | "char" => format_simple_print(val, print_type),
                            "string" => {
                                prompt::error(format!("{} `string` unsupported for {} `register`", "[type]".magenta(), "<item>".magenta()));
                                prompt::tip_nl(format!("try using an address instead - `{}`", "help print".bold()));
                                return Ok(());
                            },
                            _ => unreachable!(),
                        };

                        prompt::success_nl(format!("{}{} = {}", "$".yellow(), reg_name.bold(), value));
                    }
                }
                MPArgument::Number(MPNumber::Immediate(imm)) => {
                    let imm = match imm {
                        MPImmediate::I16(imm) => {
                            imm as u32
                        }
                        MPImmediate::I32(imm) => {
                            imm as u32
                        }
                        MPImmediate::LabelReference(label) => {
                            binary.get_label(&label)
                                    .map_err(|_| CommandError::UnknownLabel { label: label.to_string() })?
                        }
                    };

                    let map_err = |_err| CommandError::UninitialisedPrint { addr: imm };

                    let value = match print_type {
                        "byte" => format!("{}", runtime.state().get_byte(imm).map_err(map_err)?),
                        "half" => format!("{}", runtime.state().get_half(imm).map_err(map_err)?),
                        "word" => format!("{}", runtime.state().get_word(imm).map_err(map_err)?),
                        "xbyte" => format!("0x{:02x}", runtime.state().get_byte(imm).map_err(map_err)? as u8),
                        "xhalf" => format!("0x{:04x}", runtime.state().get_half(imm).map_err(map_err)? as u16),
                        "xword" | "hex" => format!("0x{:08x}", runtime.state().get_word(imm).map_err(map_err)? as u32),
                        "char" => format!("\'{}\'", ascii::escape_default((runtime.state().get_byte(imm).map_err(map_err)? & 0xFF) as u8)),
                        "string" => {
                            let mut text = String::new();

                            let mut addr = imm;
                            loop {
                                let chr =
                                    match runtime.state().get_byte(addr) {
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

            Ok(())
        }
    )
}

fn format_simple_print(val: i32, print_type: &str) -> String {
    match print_type {
        "byte" => format!("{}", val & 0xFF),
        "half" => format!("{}", val & 0xFFFF),
        "word" => format!("{}", val),
        "xbyte" => format!("0x{:02x}", (val as u32) & 0xFF),
        "xhalf" => format!("0x{:04x}", (val as u32) & 0xFFFF),
        "xword" | "hex" => format!("0x{:08x}", val as u32),
        "char"  => format!("\'{}\'", ascii::escape_default((val & 0xFF) as u8)),
        _ => unreachable!(),
    }
}
