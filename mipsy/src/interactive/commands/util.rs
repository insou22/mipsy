use crate::KTEXT_BOT;
use crate::interactive::{error::{CommandError, CommandResult}, prompt};
use colored::*;
use mipsy_lib::{Binary, decompile::Decompiled, InstSet, decompile::decompile_inst_into_parts};

pub(crate) fn expect_u32<F>(command: &str, name: &str, arg: &str, neg_tip: Option<F>) -> CommandResult<u32>
where
    F: Fn(i32) -> String
{
    match arg.parse::<u32>() {
        Ok(num) => Ok(num),
        Err(_)  => Err({
            let err = CommandError::ArgExpectedU32 { arg: name.to_string(), instead: arg.to_string() };

            match (arg.parse::<i32>(), neg_tip) {
                (Ok(neg), Some(f)) => 
                    CommandError::WithTip { 
                        error: Box::new(err), 
                        tip: f(neg),
                    },
                _ => 
                    CommandError::WithTip {
                        error: Box::new(err),
                        tip: format!("try `{} {}`", "help".bold(), command.bold()),
                    },
            }
        }),
    }
}

pub(crate) fn print_inst_parts(binary: &Binary, parts: &Decompiled, file: Option<&str>, highlight: bool) {
    if parts.inst_name.is_none() {
        return;
    }

    let name = parts.inst_name.as_ref().unwrap();

    if !parts.labels.is_empty() {
        println!();
    }

    for label in parts.labels.iter() {
        prompt::banner_nl(label.yellow().bold());
    }

    let last_line = get_last_line(binary, parts.addr);

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
        if highlight {
            format!("0x{:08x}", parts.addr).green()
        } else {
            format!("0x{:08x}", parts.addr).bright_black()
        },
        match parts.line_num {
            Some(num) => format!("{:<3}", num),
            None      => {
                if parts.addr >= KTEXT_BOT {
                    "kernel".yellow().bold().to_string()
                } else {
                    format!("{:3}", " ".repeat(last_line.to_string().len()))
                }
            }
        }.yellow().bold(),
        format!("0x{:08x}", parts.opcode).green(),
        name.yellow().bold(),
        args,
    );

    let mut line_part = String::new();
    if let Some(line_num) = parts.line_num {
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

    println!("{:60}{}", decompiled_part, line_part);
}

pub(crate) fn print_inst(iset: &InstSet, binary: &Binary, inst: u32, addr: u32, file: Option<&str>) {
    let parts = decompile_inst_into_parts(binary, iset, inst, addr);
    print_inst_parts(binary, &parts, file, false);
}

pub(crate) fn get_last_line(binary: &Binary, addr: u32) -> u32 {
    let mut last_line = 1;

    let mut lines = binary.line_numbers.iter().collect::<Vec<_>>();
    lines.sort_by_key(|&(addr, _line)| addr);

    for (&prev_addr, &prev_line) in lines {
        if prev_addr >= addr {
            break;
        }

        last_line = prev_line;
    }

    last_line
}