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

pub(crate) fn print_inst_parts(parts: &Decompiled) {
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

                while let Some(chr) = reg_chars.next() {
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

    println!(
        "{} [{}]    {:6} {}",
        format!("0x{:08x}", parts.addr).bright_black(),
        format!("0x{:08x}", parts.opcode).green(),
        name.yellow().bold(),
        args,
    );

}

pub(crate) fn print_inst(iset: &InstSet, binary: &Binary, inst: u32, addr: u32) {
    let parts = decompile_inst_into_parts(binary, iset, inst, addr);
    print_inst_parts(&parts);
}