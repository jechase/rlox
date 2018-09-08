#![allow(dead_code)]

use crate::*;

#[derive(Debug, Clone, Copy)]
pub enum TokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

#[derive(Clone, Debug)]
pub struct Token {
    ty:      TokenType,
    lexeme:  String,
    literal: Value,
    line:    usize,
}

impl Token {
    pub fn new(
        ty: TokenType,
        lexeme: String,
        literal: Value,
        line: usize,
    ) -> Token {
        Token {
            ty,
            lexeme,
            literal,
            line,
        }
    }
}
