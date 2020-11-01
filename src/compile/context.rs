use std::collections::HashMap;
use std::iter::Peekable;
use std::default::Default;
use crate::util::Safe;

pub type Address = u32;


#[derive(Debug, Clone)]
pub enum Token {
    LabelReference(String),
    Instruction(String),
    Label(String),
    Register(String),
    OffsetRegister(String),
    PlusMinus(i8),
    Comma,
    Directive(String),
    Word(i32),
    Immediate(i16),
    Float(f64),
    ConstStr(String),
    ConstChar(char),
    LineNumber(usize),
    EOF,
}

#[derive(Clone, Debug, Default)]
pub struct Program {
    pub text: Vec<u32>,
    pub data: Vec<Safe<u8>>,
    pub labels: HashMap<String, Address>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Segment {
    Text,
    Data,
    // TODO: KText, Kdata
}

#[derive(Clone, Debug)]
pub struct Context {
    tokens: Peekable<Tokens>,
    tokens_clone: Peekable<Tokens>, // do not modify
    pub program: Program,
    pub seg: Segment,
    pub line: usize,
}

impl Context {
    pub fn new(tokens: Vec<Token>) -> Self {
        let tokens = Tokens {
            tokens,
            curr: 0,
        }
        .peekable();

        Context {
            tokens_clone: tokens.clone(),
            tokens,
            program: Program::default(),
            seg: Segment::Text,
            line: 0,
        }
    }

    pub fn next_token(&mut self) -> Option<Token> {
        self.tokens.next()
    }

    pub fn peek_token(&mut self) -> Option<&Token> {
        self.tokens.peek()
    }

    pub fn next_useful_token(&mut self) -> Option<Token> {
        while let Some(token) = self.next_token() {
            if is_useful_token(&token) {
                return Some(token);
            }

            if is_newline_token(&token) {
                self.line += 1;
            }
        }

        None
    }

    pub fn peek_useful_token(&mut self) -> Option<Token> {
        while let Some(token) = self.peek_token() {
            if is_useful_token(token) {
                return Some(token.clone());
            }

            if is_newline_token(token) {
                self.line += 1;
            }

            self.next_token();
        }

        None
    }

    pub fn reset_state(&mut self) {
        self.tokens = self.tokens_clone.clone();
        self.line = 0;
        self.seg = Segment::Text;
    }
}

#[derive(Clone, Debug)]
struct Tokens {
    tokens: Vec<Token>,
    curr: usize,
}

impl Iterator for Tokens {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.tokens.get(self.curr) {
            Some(token) => {
                self.curr += 1;
                Some(token.clone())
            }
            None => None,
        }
    }
}

fn is_useful_token(token: &Token) -> bool {
    ! matches!(token, Token::Comma | Token::LineNumber(_))
}

fn is_newline_token(token: &Token) -> bool {
    matches!(token, Token::LineNumber(_))
}
