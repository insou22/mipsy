use colored::Colorize;
use mipsy_utils::MipsyConfig;
use serde::{Deserialize, Serialize};
use std::{path::MAIN_SEPARATOR, rc::Rc};

#[derive(Debug, Deserialize, Serialize)]
pub struct ParserError {
    error: Error,
    file_tag: Rc<str>,
    line: u32,
    col: u32,
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

    // TODO(zkol): Can't just pull tab_size from the config, since
    // file may have #![tabsize(...)]
    pub fn show_error(&self, config: &MipsyConfig, file: Rc<str>) {
        let message = "failed to parse".bright_red().bold();

        let line = {
            let target_line = (self.line - 1) as usize;

            let line = file.lines().nth(target_line);

            // special case: file is empty and ends with a newline, in which case the
            // parser will point to char 1-1 of the final line, but .lines() won't consider
            // that an actual line, as it doesn't contain any actual content.
            //
            // the only way this can actually occur is if the file contains no actual items,
            // as otherwise it would be happy to reach the end of the file, and return the
            // program. so we can just give a customised error message instead.
            if line.is_none() && file.ends_with('\n') && target_line == file.lines().count() {
                eprintln!("file contains no MIPS contents!");
                return;
            }

            line.expect("invalid line position in compiler error")
        };

        let updated_line = {
            let mut updated_line = String::new();

            for (idx, char) in line.char_indices() {
                if char != '\t' {
                    updated_line.push(char);
                    continue;
                }

                let spaces_to_insert = config.tab_size - (idx as u32 % config.tab_size);
                updated_line.push_str(&" ".repeat(spaces_to_insert as usize));
            }

            updated_line
        };

        let col_end = updated_line.len() as u32 + 1;

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

                format!(
                    "{}{}{}",
                    dot_slash.bold(),
                    self.file_tag.bold(),
                    line_col.bold()
                )
            }
        };
        let bar = "|".bright_blue().bold();
        let line = updated_line;
        let pre_highlight_space = " ".repeat((self.col - 1) as usize);
        let highlight = "^"
            .repeat((col_end - self.col) as usize)
            .bright_red()
            .bold();

        // and this is where the magic happens...

        if !file_name.is_empty() {
            eprintln!("{}{} {}", line_num_blank, arrow, file_name);
        }

        eprintln!("{} {}", line_num_blank, bar);
        eprintln!("{} {} {}", line_num_str_colored, bar, line);
        eprintln!(
            "{} {} {}{} {}",
            line_num_blank, bar, pre_highlight_space, highlight, message
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    ParseFailure,
}
