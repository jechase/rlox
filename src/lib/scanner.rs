use crate::*;

use std::{
    collections::HashMap,
    iter::FromIterator,
    str::FromStr,
};

use lazy_static::lazy_static;

#[derive(Debug)]
pub struct Scanner {
    source:       LoxStr,
    eof_returned: bool,
    start:        usize,
    current:      usize,
    line:         usize,
}

lazy_static! {
    static ref RESERVED_WORDS: HashMap<String, TokenType> = HashMap::from_iter(
        [
            ("and", TokenType::And),
            ("class", TokenType::Class),
            ("else", TokenType::Else),
            ("false", TokenType::False),
            ("for", TokenType::For),
            ("fun", TokenType::Fun),
            ("if", TokenType::If),
            ("nil", TokenType::Nil),
            ("or", TokenType::Or),
            ("print", TokenType::Print),
            ("return", TokenType::Return),
            ("super", TokenType::Super),
            ("this", TokenType::This),
            ("true", TokenType::True),
            ("var", TokenType::Var),
            ("while", TokenType::While),
        ]
            .iter()
            .map(|(s, v)| ((*s).into(), *v))
    );
}

impl Scanner {
    pub fn new<S>(source: S) -> Scanner
    where
        S: Into<LoxStr>,
    {
        let source = source.into();

        Scanner {
            source,
            eof_returned: false,
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn next_token(&mut self) -> Option<Result<Token, LoxError>> {
        if self.is_at_end() && self.eof_returned {
            return None;
        }

        Some(self.scan_token())
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) -> Result<Token, LoxError> {
        loop {
            if self.is_at_end() {
                self.eof_returned = true;
                return Ok(Token::new(
                    TokenType::Eof,
                    "",
                    Value::Nil,
                    self.line,
                ));
            }
            self.start = self.current;
            let ch = self.advance();
            let token = match ch {
                '(' => self.build_token(TokenType::LeftParen, None),
                ')' => self.build_token(TokenType::RightParen, None),
                '{' => self.build_token(TokenType::LeftBrace, None),
                '}' => self.build_token(TokenType::RightBrace, None),
                ',' => self.build_token(TokenType::Comma, None),
                '.' => self.build_token(TokenType::Dot, None),
                '-' => self.build_token(TokenType::Minus, None),
                '+' => self.build_token(TokenType::Plus, None),
                ';' => self.build_token(TokenType::Semicolon, None),
                '*' => self.build_token(TokenType::Star, None),
                '!' if self.peek() == '=' => {
                    self.advance();
                    self.build_token(TokenType::BangEqual, None)
                },
                '!' => self.build_token(TokenType::Bang, None),
                '=' if self.peek() == '=' => {
                    self.advance();
                    self.build_token(TokenType::EqualEqual, None)
                },
                '=' => self.build_token(TokenType::Equal, None),
                '>' if self.peek() == '=' => {
                    self.advance();
                    self.build_token(TokenType::GreaterEqual, None)
                },
                '>' => self.build_token(TokenType::Greater, None),
                '<' if self.peek() == '=' => {
                    self.advance();
                    self.build_token(TokenType::LessEqual, None)
                },
                '<' => self.build_token(TokenType::Less, None),
                '/' if self.peek() == '/' => {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                    continue;
                },
                '/' => self.build_token(TokenType::Slash, None),
                ' ' | '\t' | '\r' => continue,
                '\n' => {
                    self.line += 1;
                    continue;
                },
                '"' => self.string()?,
                c if is_digit(c) => self.number(),
                c if is_alpha(c) => self.identifier(),
                c => {
                    return Err(LoxError::scan(
                        self.line,
                        format!("unexpected character: {:?}", c as char),
                    ));
                },
            };
            return Ok(token);
        }
    }

    fn advance(&mut self) -> char {
        let current_char = self.source[self.current..].chars().nth(0).unwrap();
        self.current += current_char.len_utf8();
        current_char
    }

    fn build_token<V>(&mut self, ty: TokenType, literal: V) -> Token
    where
        V: Into<Option<Value>>,
    {
        let text = self
            .source
            .subtendril(self.start as u32, (self.current - self.start) as u32);
        let literal = literal.into().unwrap_or(Value::Nil);
        Token::new(ty, text, literal, self.line)
    }

    fn peek(&self) -> char {
        self.source[self.current..].chars().nth(0).unwrap_or('\0')
    }

    fn peek_next(&self) -> char {
        self.source[self.current..].chars().nth(1).unwrap_or('\0')
    }

    fn string(&mut self) -> Result<Token, LoxError> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(LoxError::scan(self.line, "unterminated string"));
        }

        self.advance();

        let value = self.source.subtendril(
            self.start as u32 + 1,
            (self.current - self.start) as u32 - 2,
        );

        Ok(self.build_token(TokenType::String, Value::String(value)))
    }

    fn number(&mut self) -> Token {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == '.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        let text = self.source.get(self.start..self.current).unwrap();
        self.build_token(
            TokenType::Number,
            Value::Number(
                f64::from_str(text).expect(&format!("parse float: {}", text)),
            ),
        )
    }

    fn identifier(&mut self) -> Token {
        while is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text = self.source.get(self.start..self.current).unwrap();

        let ty =
            RESERVED_WORDS.get(text).cloned().unwrap_or(TokenType::Identifier);

        self.build_token(ty, None)
    }
}

impl Iterator for Scanner {
    type Item = Result<Token, LoxError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

fn is_digit(c: char) -> bool {
    c >= '0' && c <= '9'
}

fn is_alpha(c: char) -> bool {
    (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || c == '_'
}

fn is_alpha_numeric(c: char) -> bool {
    is_digit(c) || is_alpha(c)
}

pub fn scan(input: &str) -> impl Iterator<Item = Result<Token, LoxError>> {
    Scanner::new(input)
}
