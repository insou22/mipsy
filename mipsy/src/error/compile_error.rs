use colored::*;
use mipsy_lib::CompileError;
use mipsy_lib::ArgumentType;

use crate::interactive::prompt;

fn color_arg(arg: &ArgumentType) -> String {
    match arg {
        ArgumentType::Rd | ArgumentType::Rs | ArgumentType::Rt => format!("{}{}", "$".yellow(), &arg.to_string()[1..].bold()),
        ArgumentType::J => format!("{}", arg.to_string().yellow().bold()),
        ArgumentType::OffRs | ArgumentType::OffRt => format!("i16({}{})", "$".yellow(), &arg.to_string()[5..].bold()),
        ArgumentType::Off32Rs | ArgumentType::Off32Rt => format!("i32({}{})", "$".yellow(), &arg.to_string()[5..].bold()),
        _ => arg.to_string(),
    }
}

fn highlight_line(file: &str, line: Option<u32>, col: Option<u32>, col_end: Option<u32>) {
    let (line_num, line_text) = match line {
        Some(line) => {
            let line_num = line.to_string();
            let line_text = file.lines().nth((line - 1) as usize).unwrap();

            (line_num, line_text)
        }
        None => {
            let line_text = file.lines().next().unwrap();

            (String::new(), line_text)
        }
    };

    let line_num_width = line_num.len();

    eprintln!(
        "{} {}",
        " ".repeat(line_num_width),
        "|".bright_blue().bold()
    );
    eprintln!(
        "{} {} {}",
        line_num.bright_blue().bold(),
        "|".bright_blue().bold(),
        line_text
    );

    let space_width = col.map(|col| col - 1).unwrap_or(0) as usize;
    let arrow_width = col_end
        .and_then(|end| col.map(|col| (end - col) as usize))
        .unwrap_or(line_text.len() - space_width);

    eprint!(
        "{} {} {}{} ",
        " ".repeat(line_num_width),
        "|".bright_blue().bold(),
        " ".repeat(space_width),
        "^".repeat(arrow_width).bright_red().bold()
    );
}

pub fn handle(
    error: CompileError,
    file: &str,
    line: Option<u32>,
    col: Option<u32>,
    col_end: Option<u32>,
) {
    match error {
        // users should never see these:
        CompileError::YamlMissingFunct(_) => unreachable!(),
        CompileError::YamlMissingOpcode(_) => unreachable!(),
        CompileError::MultipleMatchingInstructions(_) => unreachable!(),

        CompileError::ParseFailure { line, col } => {
            highlight_line(file, Some(line), Some(col as u32), None);
            eprintln!("{}", "failed to parse".bright_red().bold());
        }

        CompileError::NumRegisterOutOfRange(reg_index) => {
            highlight_line(file, line, col, col_end);

            eprintln!(
                "{}{}{}",
                "unknown register ".bright_red().bold(),
                "$".yellow().bold(),
                reg_index.to_string().bold()
            );
            prompt::tip_nl(format!(
                "try using a register between {0}0 and {0}31",
                "$".yellow().bold()
            ));
        }

        CompileError::NamedRegisterOutOfRange {
            reg_name,
            reg_index,
        } => {
            highlight_line(file, line, col, col_end);

            eprintln!(
                "{}{}{}{}",
                "unknown register ".bright_red().bold(),
                "$".yellow().bold(),
                reg_name.to_string().bold(),
                reg_index.to_string().bold()
            );

            let top = match reg_name {
                'v' => 1,
                'a' => 3,
                't' => 9,
                's' => 7,
                'k' => 1,
                _ => return,
            };

            prompt::tip_nl(format!(
                "try using a register between {0}{1}{3} and {0}{1}{2}",
                "$".yellow().bold(),
                reg_name.to_string().bold(),
                top.to_string().bold(),
                "0".bold(),
            ));
        }

        CompileError::UnknownRegister(name) => {
            highlight_line(file, line, col, col_end);
            eprintln!(
                "{}{}{}",
                "unknown register ".bright_red().bold(),
                "$".yellow().bold(),
                name.to_string().bold()
            );
        }

        CompileError::UnknownInstruction(inst) => {
            highlight_line(file, line, col, col_end);
            eprintln!(
                "{}`{}`",
                "unknown instruction ".bright_red().bold(),
                inst.name().bright_red().bold()
            );
        }

        CompileError::InstructionBadFormat(inst, mut matches) => {
            highlight_line(file, line, col, col_end);
            eprintln!(
                "{}`{}`{}",
                "instruction ".bright_red().bold(),
                inst.name().bold(),
                " exists but was given incorrect arguments"
                    .bright_red()
                    .bold()
            );

            prompt::tip(format!("valid formats for `{}`:", inst.name().bold()));

            matches.sort_by_key(|inst| inst.sigref().name().to_string());

            for inst in matches {
                let sigref = inst.sigref();
                let sig = sigref.compile_sig();

                eprint!("     - {:5} ", inst.sigref().name().bold());
                let args = sig
                    .format
                    .iter()
                    .enumerate()
                    .map(|(i, arg)| if sig.relative_label && i == sig.format.len() - 1 {
                        "label".yellow().bold().to_string()
                    } else {
                        color_arg(arg)
                    })
                    .collect::<Vec<String>>()
                    .join(", ");

                eprintln!("{}", args);
            }
            println!();
        }

        CompileError::InstructionSimName(inst, mut matches) => {
            highlight_line(file, line, col, col_end);
            eprintln!(
                "{}`{}`",
                "unknown instruction ".bright_red().bold(),
                inst.name().bright_red().bold()
            );
            prompt::tip("instruction(s) with a similar name exist!");

            matches.sort_by_key(|inst| inst.sigref().name().to_string());

            eprintln!("     try:");
            for inst in matches {
                let sigref = inst.sigref();
                let sig = sigref.compile_sig();

                eprint!("     - {:5} ", inst.sigref().name().bold());
                let args = sig
                    .format
                    .iter()
                    .enumerate()
                    .map(|(i, arg)| if sig.relative_label && i == sig.format.len() - 1 {
                        "label".yellow().bold().to_string()
                    } else {
                        color_arg(arg)
                    })
                    .collect::<Vec<String>>()
                    .join(", ");

                eprintln!("{}", args);
            }
            println!();
        }

        CompileError::UnresolvedLabel(label, similar) => {
            if label == "main" {
                eprintln!(
                    "\ncan't find label `{}` in program",
                    label.bright_red().bold()
                );
                prompt::tip(format!("you are required to add a `{}` label to your program", "main".bold()));
                return;
            }

            highlight_line(file, line, col, col_end);
            eprintln!(
                "{}`{}`",
                "can't find label ".bright_red().bold(),
                label.bright_red().bold()
            );

            if !similar.is_empty() {
                prompt::tip("label(s) with a similar name exist:");

                for label in similar {
                    eprintln!("     - {} ", label);
                }
            }

            println!();
        }
    }
}
