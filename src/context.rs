use crate::lexer::Token;
use crate::types::*;
use std::collections::HashMap;
use std::fmt::Display;
use std::iter::Peekable;
use std::slice::Iter;

#[derive(Clone, Debug, Default)]
pub struct Program {
    pub text: Vec<Instruction>,
    pub data: Vec<u8>,
    pub labels: HashMap<String, Address>,
}

#[derive(Clone, Debug, Display, PartialEq)]
pub enum Segment {
    Text,
    Data,
    // TODO: KText, Kdata
}

#[derive(Clone)]
pub struct Context<'a> {
    tokens: Peekable<Tokens<'a>>,
    tokens_clone: Peekable<Tokens<'a>>, // do not modify
    pub program: Program,
    pub seg: Segment,
    pub line: usize,
}

impl<'a> Context<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        let tokens = Tokens {
            iter: tokens.iter(),
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

    pub fn next_token(&mut self) -> Option<&'a Token> {
        self.tokens.next()
    }

    pub fn peek_token(&mut self) -> Option<&'a Token> {
        self.tokens.peek().copied()
    }

    pub fn next_useful_token(&mut self) -> Option<&'a Token> {
        while let Some(token) = self.next_token() {
            if is_useful_token(token) {
                return Some(token);
            }

            if is_newline_token(token) {
                self.line += 1;
            }
        }

        None
    }

    pub fn peek_useful_token(&mut self) -> Option<&'a Token> {
        while let Some(token) = self.peek_token() {
            if is_useful_token(token) {
                return Some(token);
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
struct Tokens<'a> {
    iter: Iter<'a, Token>,
}

impl<'a> Iterator for Tokens<'a> {
    type Item = &'a Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

fn is_useful_token(token: &Token) -> bool {
    ! matches!(token, Token::Comma | Token::LineNumber(_))
}

fn is_newline_token(token: &Token) -> bool {
    matches!(token, Token::LineNumber(_))
}
