use mipsy_lib::CompilerError;

pub fn generate_highlighted_line(file: String, err: &CompilerError) -> String {

        let line = &file.lines()
            .nth((err.line() - 1) as usize)
            .expect("invalid line position in compiler error");


        let updated_line = {
            let mut updated_line = String::new();
        
            for (idx, char) in line.char_indices() {
                if char != '\t' {
                    updated_line.push(char);
                    continue;
                }

                let spaces_to_insert = 8 - (idx as u32 % 8);
                updated_line.push_str(&" ".repeat(spaces_to_insert as usize));
            }

            updated_line
        };

        let line_num_str = err.line().to_string();
        let line_num_width = line_num_str.len();
        let line_num_blank = " ".repeat(line_num_width);
        

        // let bar = "|".bright_blue().bold();
        let bar = "|";
        let line = updated_line;
        let pre_highlight_space = " ".repeat((err.col() - 1) as usize);
        //            let highlight = "^".repeat((err.col_end() - err.col()) as usize).bright_red().bold();
        let highlight = "^".repeat((err.col_end() - err.col()) as usize);
        
        format!("{} {}\n{} {} {} \n{} {} {}{}", line_num_blank, bar, line_num_str, bar, line, line_num_blank, bar, pre_highlight_space, highlight)
}
