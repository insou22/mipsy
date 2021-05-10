use std::{path::MAIN_SEPARATOR, rc::Rc};

use colored::Colorize;

const TAB_SIZE: u32 = 4;

#[derive(Debug)]
pub struct ParserError {
    error:    Error,
    file_tag: Rc<str>,
    line: u32,
    col:  u32,
}

impl ParserError {
    pub fn new(error: Error, file_tag: Rc<str>, line: u32, col: u32) -> Self {
        Self {
            error,
            file_tag,
            line,
            col,
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

    pub fn show_error(&self, file: Rc<str>) {
        let message = "failed to parse";
        
        let line = file.lines()
            .nth((self.line - 1) as usize)
            .expect("invalid line position in compiler error");

        let (updated_line, untabbed_col) = {
            let mut updated_line = String::new();
            let mut untabbed_col = self.col;
            
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

            }

            (updated_line, untabbed_col)
        };

        // format of the error:

        //   --> ./foo.s:1:2
        //    |
        // 22 | mips c'ode here
        //    |       ^ error: failed to parse this
        //

        let line_num_str = line.to_string();
        let line_num_width = line_num_str.len();
        let line_num_blank = " ".repeat(line_num_width);
        let arrow = "-->".bright_blue().bold();
        let file_name = {
            if self.file_tag.is_empty() {
                String::new()
            } else if self.file_tag.contains(MAIN_SEPARATOR) {
                self.file_tag.bold().to_string()
            } else {
                format!("./{}", self.file_tag).bold().to_string()
            }
        };
        let bar = "|".bright_blue().bold();
        let line = updated_line;
        let pre_highlight_space = untabbed_col;
        let highlight = "^".bright_red().bold();

        // and this is where the magic happens...

        if !file_name.is_empty() {
            println!("{} {} {}", line_num_blank, arrow, file_name);
        }

        println!("{} {}", line_num_blank, bar);
        println!("{} {} {}", line_num_str, bar, line);
        print!  ("{} {} {}{} {}", line_num_blank, bar, pre_highlight_space, highlight, message);
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    ParseFailure,
}
