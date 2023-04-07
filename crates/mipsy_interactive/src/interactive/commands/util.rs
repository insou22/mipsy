use mipsy_lib::{KTEXT_BOT, decompile::Uninit};
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

pub(crate) fn print_inst_parts(binary: &Binary, parts: &Result<Decompiled, Uninit>, files: Option<&[(String, String)]>, highlight: bool) {
    let labels = match parts {
        Ok(ok)      => &ok.labels,
        Err(uninit) => &uninit.labels,
    };

    if !labels.is_empty() {
        println!();
    }

    for label in labels.iter() {
        prompt::banner_nl(label.yellow().bold());
    }

    if let Err(parts) = parts {
        let last_line_len = get_final_line(binary).to_string().len();

        println!(
            "{} {} [{}]",
            if highlight {
                format!("0x{:08x}", parts.addr).green()
            } else {
                format!("0x{:08x}", parts.addr).bright_black()
            },
            if parts.addr >= KTEXT_BOT {
                "kernel".yellow().bold().to_string()
            } else {
                match parts.location {
                    Some((_, num)) => format!("{:<last_line_len$}", num),
                    None           => format!("{:last_line_len$}", "")
                }
            }.yellow().bold(),
            "uninitialised".red(),
        );

        return;
    }

    let parts = parts.as_ref().expect("just checked Err case");
    
    if parts.inst_name.is_none() {
        return;
    }

    let name = parts.inst_name.as_ref().unwrap();

    let last_line_len = get_final_line(binary).to_string().len();

    let args = parts.arguments
        .iter()
        .map(|arg| {
            if let Some(index) = arg.chars().position(|chr| chr == '$') {
                let before = arg.chars().take(index).collect::<String>();

                let mut reg_name   = String::new();
                let mut post_chars = String::new();

                let reg_chars = arg.chars().skip(index + 1);
                for chr in reg_chars.clone() {
                    if chr.is_alphanumeric() {
                        reg_name.push(chr);
                    } else {
                        post_chars.push(chr);
                        break;
                    }
                }

                // for chr in reg_chars {
                //     post_chars.push(chr);
                // }

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
        if parts.addr >= KTEXT_BOT {
            "kernel".yellow().bold().to_string()
        } else {
            match parts.location {
                Some((_, num)) => format!("{:<last_line_len$}", num),
                None           => format!("{:last_line_len$}", "")
            }
        }.yellow().bold(),
        format!("0x{:08x}", parts.opcode).green(),
        name.yellow().bold(),
        args,
    );

    let mut line_part = String::new();
    if let Some((file_name, line_num)) = parts.location.clone() {
        let file = files
            .and_then(|files| 
                files.iter()
                    .filter(|(tag, _)| tag == &*file_name)
                    .map(|(_, file)| file)
                    .next()
            );

        if let Some(file) = file {
            if let Some(line) = file.lines().nth((line_num - 1) as usize) {
                let repeat_space = {
                    let chars = strip_ansi_escapes::strip(&decompiled_part).unwrap().len();

                    60_usize.saturating_sub(chars)
                };

                line_part = format!("{} {}  {}", " ".repeat(repeat_space), "#".bright_black(), line.trim().bright_black())
            }
        }
    }

    println!("{decompiled_part}{line_part}");
}

pub(crate) fn print_inst(iset: &InstSet, binary: &Binary, inst: u32, addr: u32, files: Option<&[(String, String)]>) {
    let parts = decompile_inst_into_parts(binary, iset, inst, addr);
    print_inst_parts(binary, &Ok(parts), files, false);
}

pub(crate) fn get_final_line(binary: &Binary) -> u32 {
    binary.line_numbers.iter()
        .map(|(_, (_, line))| line)
        .max()
        .copied()
        .unwrap_or(1)
}
