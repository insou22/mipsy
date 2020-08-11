use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone)]
struct ParseContext<'a> {
    tokens: Vec<Token>,
    chars: Peekable<Chars<'a>>,
    curr_line: usize,
}

#[derive(Debug, Clone)]
pub enum Token {
    Text(String),
    Instruction(String),
    Label(String),
    Register(String),
    OffsetRegister(String),
    Comma,
    Directive(String),
    Number(isize),
    Float(f32),
    ConstStr(String),
    ConstChar(char),
    LineNumber(usize),
}

pub fn tokenise(program: &str) -> Result<Vec<Token>, String> {
    let mut context = ParseContext {
        tokens: vec![Token::LineNumber(1)],
        chars: program.chars().peekable(),
        curr_line: 1,
    };

    loop {
        if let None = context.chars.peek() {
            break;
        } else if let Some(string) = match_string(&mut context, false) {
            match context.tokens.last() {
                Some(Token::LineNumber(_)) | None => {
                    context.tokens.push(Token::Instruction(string));
                }
                _ => {
                    context.tokens.push(Token::Text(string));
                }
            }
        } else if let Some(()) = match_label(&mut context) {
            match context.tokens.last() {
                Some(Token::Text(string)) | Some(Token::Instruction(string)) => {
                    let mut new_token = Token::Label(string.clone());
                    std::mem::swap(context.tokens.last_mut().unwrap(), &mut new_token);
                }
                _ => {
                    return Err(format!(
                        "No identifier for label on line {}",
                        context.curr_line
                    ));
                }
            }
        } else if let Some(string) = match_register(&mut context) {
            context.tokens.push(Token::Register(string))
        } else if let Some(string) = match_offset_register(&mut context) {
            context.tokens.push(Token::OffsetRegister(string));
        } else if let Some(()) = match_comma(&mut context) {
            context.tokens.push(Token::Comma);
        } else if let Some(string) = match_directive(&mut context) {
            context.tokens.push(Token::Directive(string));
        } else if let Some(num) = match_num(&mut context) {
            context.tokens.push(Token::Number(num));
        } else if let Some(float) = match_float(&mut context) {
            context.tokens.push(Token::Float(float));
        } else if let Some(string) = match_const_str(&mut context) {
            context.tokens.push(Token::ConstStr(string));
        } else if let Some(chr) = match_const_char(&mut context) {
            context.tokens.push(Token::ConstChar(chr));
        } else if let Some(()) = match_comment(&mut context) {
            // ...
        } else if let Some(nl) = match_whitespace(&mut context) {
            for _ in 0..nl {
                context.curr_line += 1;
                context.tokens.push(Token::LineNumber(context.curr_line));
            }
        } else {
            return Err(format!(
                "Failed to parse line {}, near character: '{}'",
                context.curr_line,
                context.chars.peek().unwrap()
            ));
        }
    }

    Ok(context.tokens)
}

fn match_string(context: &mut ParseContext, start_num: bool) -> Option<String> {
    let mut string = String::new();

    while let Some(&chr) = context.chars.peek() {
        match chr {
            '0'..='9' => {
                if string.is_empty() && !start_num {
                    return None;
                }

                string.push(chr);
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                string.push(chr);
            }
            _ => break,
        }

        context.chars.next();
    }

    if string.is_empty() {
        return None;
    }

    Some(string)
}

fn match_offset_register(context: &mut ParseContext) -> Option<String> {
    let mut context_clone = context.clone();

    if let Some(chr) = context_clone.chars.next() {
        if chr != '(' {
            return None;
        }
    } else {
        return None;
    }

    if let Some(nl) = match_whitespace(&mut context_clone) {
        for _ in 0..nl {
            context_clone.curr_line += 1;
            context_clone
                .tokens
                .push(Token::LineNumber(context_clone.curr_line));
        }
    }

    if let Some(reg) = match_register(&mut context_clone) {
        if let Some(chr) = context_clone.chars.next() {
            if chr != ')' {
                return None;
            }

            std::mem::swap(context, &mut context_clone);
            return Some(reg);
        }

        return None;
    }

    None
}

fn match_register(context: &mut ParseContext) -> Option<String> {
    let mut context_clone = context.clone();

    if let Some(&chr) = context_clone.chars.peek() {
        if chr != '$' {
            return None;
        }
        context_clone.chars.next();

        if let Some(string) = match_string(&mut context_clone, true) {
            std::mem::swap(context, &mut context_clone);
            return Some(string);
        }
    }

    None
}

fn match_label(context: &mut ParseContext) -> Option<()> {
    if let Some(&chr) = context.chars.peek() {
        if chr == ':' {
            context.chars.next();
            return Some(());
        }
    }

    None
}

fn match_comma(context: &mut ParseContext) -> Option<()> {
    if let Some(&chr) = context.chars.peek() {
        if chr == ',' {
            context.chars.next();
            return Some(());
        }
    }

    None
}

fn match_directive(context: &mut ParseContext) -> Option<String> {
    let mut context_clone = context.clone();

    if let Some(&chr) = context_clone.chars.peek() {
        if chr != '.' {
            return None;
        }
        context_clone.chars.next();

        if let Some(string) = match_string(&mut context_clone, false) {
            std::mem::swap(context, &mut context_clone);
            return Some(string);
        }
    }

    None
}

fn match_float(context: &mut ParseContext) -> Option<f32> {
    let mut iter_clone = context.chars.clone();

    let mut digits = vec![];

    if let Some(&chr) = iter_clone.peek() {
        if chr == '-' {
            iter_clone.next();
            digits.push(chr);
        }
    }

    let mut seen_dot = false;
    while let Some(&chr) = iter_clone.peek() {
        match chr {
            '0'..='9' => {
                digits.push(chr);
                iter_clone.next();
            }
            '.' => {
                if seen_dot {
                    return None;
                } else {
                    seen_dot = true;

                    digits.push('.');
                    iter_clone.next();
                }
            }
            _ => {
                if !seen_dot {
                    return None;
                }

                if let Some(result) = chars_to_f32(digits.into_iter().collect()) {
                    std::mem::swap(&mut context.chars, &mut iter_clone);
                    return Some(result);
                }

                break;
            }
        }
    }

    None
}

fn match_num(context: &mut ParseContext) -> Option<isize> {
    let mut iter_clone = context.chars.clone();

    let mut digits = vec![];
    let mut base = 10;

    if let Some(&chr) = iter_clone.peek() {
        if chr == '-' {
            iter_clone.next();
            digits.push(chr);
        }
    }

    while let Some(&chr) = iter_clone.peek() {
        match chr {
            '0' => {
                if digits.is_empty() || digits.len() == 1 && digits[0] == '-' {
                    iter_clone.next();

                    if let Some(&next) = iter_clone.peek() {
                        match next {
                            'x' | 'X' => {
                                base = 16;
                                iter_clone.next();
                            }
                            '0'..='9' => {
                                base = 8;
                            }
                            _ => {
                                std::mem::swap(&mut context.chars, &mut iter_clone);
                                return Some(0);
                            }
                        }
                    } else {
                        std::mem::swap(&mut context.chars, &mut iter_clone);
                        return Some(0);
                    }
                } else {
                    digits.push(chr);
                    iter_clone.next();
                }
            }
            '1'..='9' => {
                digits.push(chr);
                iter_clone.next();
            }
            '.' => {
                // this is a job for match_float
                return None;
            }
            _ => {
                if let Some(result) = chars_to_isize(digits.into_iter().collect(), base) {
                    std::mem::swap(&mut context.chars, &mut iter_clone);
                    return Some(result);
                }

                break;
            }
        }
    }

    None
}

fn chars_to_isize(numbers: String, base: u32) -> Option<isize> {
    if numbers.is_empty() {
        return None;
    }

    match isize::from_str_radix(&numbers, base) {
        Ok(i) => Some(i),
        Err(_) => None,
    }
}

fn chars_to_f32(numbers: String) -> Option<f32> {
    if numbers.is_empty() {
        return None;
    }

    match numbers.parse::<f32>() {
        Ok(i) => Some(i),
        Err(_) => None,
    }
}

fn match_const_str(context: &mut ParseContext) -> Option<String> {
    let mut string = String::new();

    if context.chars.peek() != Some(&'\"') {
        return None;
    }

    let mut context_clone = context.clone();
    context_clone.chars.next();

    while let Some(chr) = context_clone.chars.next() {
        match chr {
            '"' => {
                std::mem::swap(context, &mut context_clone);
                return Some(string);
            }
            '\\' => {
                if let Some(chr) = context_clone.chars.next() {
                    match chr {
                        '0' => {
                            string.push('\0');
                        }
                        'n' => {
                            string.push('\n');
                        }
                        't' => {
                            string.push('\t');
                        }
                        '\\' => {
                            string.push('\\');
                        }
                        '\"' => {
                            string.push('\"');
                        }
                        '\'' => {
                            string.push('\'');
                        }
                        _ => {
                            panic!("Unknown escape sequence: \\{}", chr);
                        }
                    }
                } else {
                    // ??????
                    panic!("File ended on backslash??");
                }
            }
            _ => {
                string.push(chr);
            }
        }
    }

    None
}

fn match_const_char(context: &mut ParseContext) -> Option<char> {
    let mut iter_clone = context.chars.clone();

    if iter_clone.next() != Some('\'') {
        return None;
    }

    let chr = iter_clone.next();

    if chr == None {
        return None;
    }

    let mut chr = chr.unwrap();

    if chr == '\\' {
        match iter_clone.next() {
            Some('0') => {
                chr = '\0';
            }
            Some('n') => {
                chr = '\n';
            }
            Some('t') => {
                chr = '\t';
            }
            Some('\\') => {
                chr = '\\';
            }
            Some('\"') => {
                chr = '\"';
            }
            Some('\'') => {
                chr = '\'';
            }
            None => {
                return None;
            }
            _ => {
                panic!("Unknown escape sequence: \\{}", chr);
            }
        }
    }

    if iter_clone.next() != Some('\'') {
        return None;
    }

    std::mem::swap(&mut context.chars, &mut iter_clone);
    Some(chr)
}

fn match_comment(context: &mut ParseContext) -> Option<()> {
    if let Some(&chr) = context.chars.peek() {
        if chr == '#' || chr == ';' {
            context.chars.next();

            while let Some(&chr) = context.chars.peek() {
                match chr {
                    '\n' => {
                        break;
                    }
                    _ => {
                        context.chars.next();
                    }
                }
            }

            return Some(());
        }
    }

    None
}

fn match_whitespace(context: &mut ParseContext) -> Option<usize> {
    let mut found = false;
    let mut newlines = 0;

    while let Some(&chr) = context.chars.peek() {
        match chr {
            ' ' | '\t' | '\r' | '\n' => {
                context.chars.next();

                found = true;
                if chr == '\n' {
                    newlines += 1;
                }
            }
            _ => {
                break;
            }
        }
    }

    if found {
        return Some(newlines);
    }

    None
}
