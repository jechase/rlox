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
    pub literal: Primitive,
    pub line:    usize,
}

impl Token {
    pub fn new<S, P>(ty: TokenType, lexeme: S, literal: P, line: usize) -> Token
    where
        S: Into<LoxStr>,
        P: Into<Primitive>,
    {
        let lexeme = lexeme.into();
        Token {
            ty,
            lexeme: lexeme.into(),
            literal: literal.into(),
            line,
        }
    }
}
