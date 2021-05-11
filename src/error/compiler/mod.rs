use std::{path::MAIN_SEPARATOR, rc::Rc};

use colored::Colorize;
use mipsy_parser::MpInstruction;
use crate::inst::instruction::Signature;

use super::util::{syntax_highlight_argument, tip_header};

const TAB_SIZE: u32 = 4;

#[derive(Debug)]
pub struct CompilerError {
    error:    Error,
    file_tag: Rc<str>,
    line:     u32,
    col:      u32,
    col_end:  u32,
}

impl CompilerError {
    pub fn new(error: Error, file_tag: Rc<str>, line: u32, col: u32, col_end: u32) -> Self {
        Self {
            error,
            file_tag,
            line,
            col,
            col_end,
        }
    }

    pub fn error(&self) -> &Error {
        &self.error
    }

    pub fn file_tag(&self) -> Rc<str> {
        self.file_tag.clone()
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    pub fn col(&self) -> u32 {
        self.col
    }

    pub fn col_end(&self) -> u32 {
        self.col_end
    }

    pub fn show_error(&self, file: Rc<str>) {
        if self.error().should_highlight_line() {
            self.highlight_line(file);
        }

        println!("{}", self.error.message());

        for tip in self.error.tips() {
            println!("{} {}", tip_header(), tip);
        }
    }

    fn highlight_line(&self, file: Rc<str>) {
        let line = file.lines()
            .nth((self.line - 1) as usize)
            .expect("invalid line position in compiler error");

        let (updated_line, untabbed_col, untabbed_col_end) = {
            let mut updated_line = String::new();
            let mut untabbed_col = self.col;
            let mut untabbed_col_end = self.col_end;    
            
            for (idx, char) in line.char_indices() {
                if char != '\t' {
                    updated_line.push(char);
                    continue;
                }

                let spaces_to_insert = TAB_SIZE - (idx as u32 % TAB_SIZE);
                updated_line.push_str(&" ".repeat(spaces_to_insert as usize));

                if idx < self.col as usize {
                    untabbed_col += spaces_to_insert - 1;
                }

                if idx < self.col_end as usize {
                    untabbed_col_end += spaces_to_insert - 1;
                }
            }

            (updated_line, untabbed_col, untabbed_col_end)
        };

        // format of the error:

        //   --> ./foo.s:1:2
        //    |
        // 22 | mips code here
        //    |      ^^^^ error: some useless diagnosis
        //

        let line_num_str = self.line.to_string();
        let line_num_str_colored = line_num_str.bright_blue().bold();
        let line_num_width = line_num_str.len();
        let line_num_blank = " ".repeat(line_num_width);
        let arrow = "-->".bright_blue().bold();
        let file_name = {
            if self.file_tag.is_empty() {
                String::new()
            } else {
                let dot_slash = if !self.file_tag.contains(MAIN_SEPARATOR) {
                    "./"
                } else {
                    ""
                };

                let line_col = format!(":{}:{}", self.line, self.col);

                format!("{}{}{}", dot_slash.bold(), self.file_tag.bold(), line_col.bold())
            }
        };
        let bar = "|".bright_blue().bold();
        let line = updated_line;
        let pre_highlight_space = " ".repeat((untabbed_col - 1) as usize);
        let highlight = "^".repeat((untabbed_col_end - untabbed_col) as usize).bright_red().bold();

        // and this is where the magic happens...

        if !file_name.is_empty() {
            println!("{}{} {}", line_num_blank, arrow, file_name);
        }

        println!("{} {}", line_num_blank, bar);
        println!("{} {} {}", line_num_str_colored, bar, line);
        print!  ("{} {} {}{} ", line_num_blank, bar, pre_highlight_space, highlight);
    }
}

#[derive(Debug)]
pub enum Error {
    NumberedRegisterOutOfRange { reg_num: i32 },
    NamedRegisterOutOfRange    { reg_name: char, reg_index: i32 },
    UnknownRegister            { reg_name: String },

    UnknownInstruction   { inst_ast: MpInstruction },
    InstructionBadFormat { inst_ast: MpInstruction, correct_formats: Vec<Signature> },
    InstructionSimName   { inst_ast: MpInstruction, similar_instns:  Vec<Signature> },

    RedefinedLabel  { label: String },
    UnresolvedLabel { label: String, similar: Vec<String> },
}

impl Error {
    pub fn message(&self) -> String {
        match self {
            Error::NumberedRegisterOutOfRange { reg_num } => {
                let message = "unknown register".bright_red().bold();
                let register_dollar = "$".yellow().bold();
                let register_num = reg_num.to_string().bold();

                format!("{} {}{}", message, register_dollar, register_num)
            }

            Error::NamedRegisterOutOfRange { reg_name, reg_index } => {
                let message = "unknown register".bright_red().bold();
                let register_dollar = "$".yellow().bold();
                let reg_name = reg_name.to_string().bold();
                let reg_index = reg_index.to_string().bold();
                
                format!("{} {}{}{}", message, register_dollar, reg_name, reg_index)
            }
            
            Error::UnknownRegister { reg_name } => {
                let message = "unknown register".bright_red().bold();
                let register_dollar = "$".yellow().bold();
                let name = reg_name.bold();

                format!("{} {}{}", message, register_dollar, name)
            }
            Error::UnknownInstruction { inst_ast } | 
            Error::InstructionSimName { inst_ast, .. } => {
                let message = "unknown instruction".bright_red().bold();
                let inst_name = inst_ast.name().bright_red().bold();

                format!("{} `{}`", message, inst_name)
            }
            
            Error::InstructionBadFormat { inst_ast, .. } => {
                let message_1 = "instruction".bright_red().bold();
                let message_2 = "exists but was given incorrect arguments".bright_red().bold();
                let inst_name = inst_ast.name().bold();

                format!("{} `{}` {}", message_1, inst_name, message_2)
            }

            Error::RedefinedLabel { label } => {
                let message_1 = "the label".bright_red().bold();
                let message_2 = "is defined multiple times".bright_red().bold();
                let label = label.bold();

                format!("{} `{}` {}", message_1, label, message_2)
            }
            
            Error::UnresolvedLabel { label, .. } => {
                let message_1 = "cannot find label".bright_red().bold();
                let message_2 = "in program".bright_red().bold();
                let label = label.bright_red().bold();
                
                format!("{} `{}` {}", message_1, label, message_2)
            }
        }
    }

    pub fn tips(&self) -> Vec<String> {
        match self {
            Error::NumberedRegisterOutOfRange { .. } => {
                let register_dollar = "$".yellow().bold();
                let min_reg = format!("{}{}", register_dollar, "0".bold());
                let max_reg = format!("{}{}", register_dollar, "31".bold());

                vec![
                    format!(
                        "try using a register between {} and {}\n",
                        min_reg,
                        max_reg,
                    )
                ]
            }

            Error::NamedRegisterOutOfRange { reg_name, .. } => {
                let register_dollar = "$".yellow().bold();
                let register_name = reg_name.to_string().bold();

                let bottom = "0".bold();
                let top = match reg_name {
                    'v' => 1,
                    'a' => 3,
                    't' => 9,
                    's' => 7,
                    'k' => 1,
                    _ => unreachable!(),
                }.to_string().bold();
    
                vec![
                    format!(
                        "try using a register between {0}{1}{2} and {0}{1}{3}\n",
                        register_dollar,
                        register_name,
                        bottom,
                        top,
                    )
                ]
            }

            Error::UnknownRegister { .. } => {
                // good luck kiddo
                vec![]
            }
            
            Error::UnknownInstruction { .. } => {
                // good luck kiddo
                vec![]
            }
            
            Error::InstructionBadFormat { inst_ast, correct_formats } => {
                let mut tip = String::new();

                let inst_name = inst_ast.name().bold();

                tip.push_str(&format!("valid formats for `{}`:\n", inst_name));

                for inst in correct_formats {
                    let sigref = inst.sigref();
                    let sig = sigref.compile_sig();

                    let inst_name = sigref.name().bold();
    
                    tip.push_str(&format!("  - {} ", inst_name));

                    let args = sig
                        .format
                        .iter()
                        .enumerate()
                        .map(|(i, arg)| {
                            // special case for relative labels
                            if sig.relative_label && i == sig.format.len() - 1 {
                                "label".yellow().bold().to_string()
                            } else {
                                syntax_highlight_argument(arg)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(", ");
    
                    tip.push_str(&args);
                    tip.push('\n');
                }
    
                vec![tip]
            }

            Error::InstructionSimName { inst_ast: _, similar_instns } => {
                let mut tip = String::new();

                tip.push_str("instruction(s) with a similar name exist:\n");

                let sorted_similar = {
                    let mut sorted_similar = similar_instns.clone();
                    sorted_similar.sort_by_key(|sig| sig.sigref().name().to_string());

                    sorted_similar
                };

                for inst in sorted_similar {
                    let sigref = inst.sigref();
                    let sig = sigref.compile_sig();

                    tip.push_str(&format!("  - {:5} ", inst.sigref().name().bold()));

                    let args = sig
                        .format
                        .iter()
                        .enumerate()
                        .map(|(i, arg)| if sig.relative_label && i == sig.format.len() - 1 {
                            "label".yellow().bold().to_string()
                        } else {
                            syntax_highlight_argument(arg)
                        })
                        .collect::<Vec<_>>()
                        .join(", ");

                    tip.push_str(&args);
                    tip.push('\n');
                }

                vec![tip]
            }

            Error::RedefinedLabel { .. } => {
                // good luck kiddo
                vec![]
            }
            
            Error::UnresolvedLabel { label, similar } => {
                if label == "main" {
                    let message_1 = "you are required to add a";
                    let message_2 = "label to your program";
                    let main = "main".bold();

                    let tip = format!("{} `{}` {}\n", message_1, main, message_2);
                    
                    vec![tip]
                } else if !similar.is_empty() {
                    let mut tip = String::new();

                    tip.push_str("label(s) with a similar name exist:\n");
                    for label in similar {
                        tip.push_str(&format!(" - {}\n", label.yellow().bold()));
                    }

                    vec![tip]
                } else {
                    vec![]
                }
            }
        }
    }

    pub fn should_highlight_line(&self) -> bool {
        match self {
            // only highlight the error-ing line if the requested label is not `main`
            Self::UnresolvedLabel { label, .. } => label != "main",

            // otherwise highlight the line causing the error
            _ => true,
        }
    }
}
