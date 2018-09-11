#![allow(dead_code)]

use crate::*;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
    pub ty:      TokenType,
    pub lexeme:  LoxStr,
    pub literal: Value,
    pub line:    usize,
}

impl Token {
    pub fn new<S>(
        ty: TokenType,
        lexeme: S,
        literal: Value,
        line: usize,
    ) -> Token
    where
        S: Into<LoxStr>,
    {
        let lexeme = lexeme.into();
        Token {
            ty,
            lexeme,
            literal,
            line,
        }
    }
}
