use mipsy_lib::{CompilerError, Binary, InstSet, decompile::decompile_into_parts};

pub fn generate_highlighted_line(file: String, err: &CompilerError) -> String {
    let line = &file
        .lines()
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

    format!(
        "{} {}\n{} {} {} \n{} {} {}{}",
        line_num_blank,
        bar,
        line_num_str,
        bar,
        line,
        line_num_blank,
        bar,
        pre_highlight_space,
        highlight
    )
}

pub fn decompile(program: &Binary, iset: &InstSet, file: Option<String>) -> String {
    let mut text = String::new();
    let unknown_instruction = String::from("# Unknown instruction");

    let decompiled = decompile_into_parts(program, iset);

    let mut keys: Vec<u32> = decompiled.keys().copied().collect();
    keys.sort_unstable();

    for (addr, parts) in keys.into_iter().map(|addr| (addr, decompiled.get(&addr).unwrap())) {
        if let Err(parts) = parts {
            if !parts.labels.is_empty() {
                text.push('\n');
            }

            for label in parts.labels.iter() {
                text.push_str(&format!("{}: \n", label));
            }

            text.push_str(&format!("0x{:08x}: [uninitialised]\n", addr));
            continue;
        }

        let parts = parts.as_ref().expect("just checked Err case");

        if !parts.labels.is_empty() {
            text.push('\n');
        }

        for label in parts.labels.iter() {
            text.push_str(&format!("{}: \n", label));
        }

        let decompiled_part = &format!(
            "0x{:08x} [0x{:08x}]    {:6} {}",
            addr,
            parts.opcode,
            parts.inst_name.as_ref().unwrap_or(&unknown_instruction),
            parts.arguments.join(", ")
        );

        text.push_str(&decompiled_part);

        if let Some(file) = file.clone() {
            if let Some((_, line_num)) = parts.location.clone() {
                if let Some(line) = file.lines().nth((line_num - 1) as usize) {
                    let repeat_space = {
                        let chars = (&decompiled_part).len();
                        60_usize.saturating_sub(chars)
                    };
                    text.push_str(&format!("{}; [{}] {}", " ".repeat(repeat_space), line_num, line.trim_start()));
                }
            }
        }

        text.push('\n');
    }
        
    text
}
