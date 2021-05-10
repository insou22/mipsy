use std::rc::Rc;

use colored::Colorize;
use crate::{ArgumentType, Binary, InstSet, KTEXT_BOT, decompile::{Decompiled, decompile_inst_into_parts}};

// arg.to_string() will simply use the existing Display impl
// eg ArgumentType::Rd.to_string() == "$Rd"
//    ArgumentType::J .to_string() == "label"
pub fn syntax_highlight_argument(arg: &ArgumentType) -> String {
    match arg {
        // register
        ArgumentType::Rd | ArgumentType::Rs | ArgumentType::Rt => {
            let register_dollar = "$".yellow();
            let argument = arg.to_string()[1..].bold();

            format!("{}{}", register_dollar, argument)
        }
        // jump label
        ArgumentType::J => {
            arg.to_string().yellow().bold().to_string()
        }
        // offset-register
        ArgumentType::OffRs | ArgumentType::OffRt => { 
            let register_dollar = "$".yellow();
            let register_name = arg.to_string()[5..].bold();

            format!("i16({}{})", register_dollar, register_name)
        }
        // offset32-register
        ArgumentType::Off32Rs | ArgumentType::Off32Rt => {
            let register_dollar = "$".yellow();
            let register_name = arg.to_string()[5..].bold();

            format!("i32({}{})", register_dollar, register_name)
        }
        // purely numeric
        ArgumentType::Shamt | ArgumentType::U16 | ArgumentType::U32 | ArgumentType::I16 | ArgumentType::I32 |
        ArgumentType::F32 | ArgumentType::F64 => {
            arg.to_string()
        }
    }
}

pub fn tip_header() -> String {
    let header = "tip".yellow().bold();
    let colon = ":".bold();

    format!("{}{}", header, colon)
}

pub fn inst_to_string(inst: u32, addr: u32, source_code: &Vec<(Rc<str>, Rc<str>)>, binary: &Binary, iset: &InstSet, highlight_curr_inst: bool) -> String {
    let parts = decompile_inst_into_parts(binary, iset, inst, addr);
    inst_parts_to_string(&parts, source_code, binary, highlight_curr_inst)
}

pub fn inst_parts_to_string(parts: &Decompiled, source_code: &Vec<(Rc<str>, Rc<str>)>, binary: &Binary, highlight_curr_inst: bool) -> String {
    let mut string = String::new();
    
    if parts.inst_name.is_none() {
        return string;
    }

    let name = parts.inst_name.as_ref().unwrap();

    if !parts.labels.is_empty() {
        string.push('\n');
    }

    for label in parts.labels.iter() {
        let label = label.yellow().bold();
        let colon = ":".bold();
        string.push_str(&format!("{}{}\n", label, colon));
    }

    let last_line_num = get_last_line_number(binary, parts.addr);

    let args = parts.arguments
        .iter()
        .map(|arg| {
            if let Some(index) = arg.chars().position(|chr| chr == '$') {
                let before = arg.chars().take(index).collect::<String>();

                let mut reg_name   = String::new();
                let mut post_chars = String::new();

                let mut reg_chars = arg.chars().skip(index + 1);
                while let Some(chr) = reg_chars.next() {
                    if chr.is_alphanumeric() {
                        reg_name.push(chr);
                    } else {
                        post_chars.push(chr);
                        break;
                    }
                }

                for chr in reg_chars {
                    post_chars.push(chr);
                }

                format!("{}{}{}{}", before, "$".yellow(), reg_name.bold(), post_chars)
            } else if arg.chars().next().unwrap().is_alphabetic() {
                arg.yellow().bold().to_string()
            } else {
                arg.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join(", ");

    let decompiled_part = format!(
        "{} {} [{}]    {:6} {}",
        if highlight_curr_inst {
            format!("0x{:08x}", parts.addr).green()
        } else {
            format!("0x{:08x}", parts.addr).bright_black()
        },
        match parts.location {
            Some((_, num)) => format!("{:<3}", num),
            None      => {
                if parts.addr >= KTEXT_BOT {
                    "kernel".yellow().bold().to_string()
                } else {
                    format!("{:3}", " ".repeat(last_line_num.to_string().len()))
                }
            }
        }.yellow().bold(),
        format!("0x{:08x}", parts.opcode).green(),
        name.yellow().bold(),
        args,
    );

    let mut line_part = String::new();

    if let Some((file_tag, line_num)) = &parts.location {
        let mut file = None;
        
        for (src_tag, src_file) in source_code {
            if *file_tag == *src_tag {
                file = Some(src_file);
                break;
            }
        }

        if let Some(file) = file {
            if let Some(line) = file.lines().nth((line_num - 1) as usize) {
                let repeat_space = {
                    let chars = strip_ansi_escapes::strip(&decompiled_part).unwrap().len();

                    if chars >= 55 {
                        0
                    } else {
                        55 - chars
                    }
                };

                line_part = format!("{} {}  {}", " ".repeat(repeat_space), "#".bright_black(), line.trim().bright_black())
            }
        }
    }

    string.push_str(&format!("{:60}{}", decompiled_part, line_part));

    string
}

pub(crate) fn get_last_line_number(binary: &Binary, addr: u32) -> u32 {
    let mut last_line = 1;

    let mut lines = binary.line_numbers.iter().collect::<Vec<_>>();
    lines.sort_by_key(|&(addr, _line)| addr);

    for (&prev_addr, (_file_name, prev_line)) in lines {
        if prev_addr >= addr {
            break;
        }

        last_line = *prev_line;
    }

    last_line
}
