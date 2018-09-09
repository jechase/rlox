use crate::*;

use std::{
    collections::HashMap,
    iter::FromIterator,
    str::FromStr,
};

#[derive(Debug)]
pub struct Scanner {
    source:   String,
    tokens:   Vec<Token>,
    start:    usize,
    current:  usize,
    line:     usize,
    reporter: Reporter,
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
    pub fn new(source: String) -> Scanner {
        Scanner {
            source,
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            reporter: Default::default(),
        }
    }

    pub fn scan_tokens(mut self) -> Result<Vec<Token>, Errors> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token();
        }

        self.tokens.push(Token::new(
            TokenType::Eof,
            "".into(),
            Value::Nil,
            self.line,
        ));

        self.reporter.finish()?;

        Ok(self.tokens)
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn scan_token(&mut self) {
        let ch = self.advance();
        match ch {
            b'(' => self.add_token(TokenType::LeftParen, None),
            b')' => self.add_token(TokenType::RightParen, None),
            b'{' => self.add_token(TokenType::LeftBrace, None),
            b'}' => self.add_token(TokenType::RightBrace, None),
            b',' => self.add_token(TokenType::Comma, None),
            b'.' => self.add_token(TokenType::Dot, None),
            b'-' => self.add_token(TokenType::Minus, None),
            b'+' => self.add_token(TokenType::Plus, None),
            b';' => self.add_token(TokenType::Semicolon, None),
            b'*' => self.add_token(TokenType::Star, None),
            b'!' if self.peek() == b'=' => {
                self.advance();
                self.add_token(TokenType::BangEqual, None)
            },
            b'!' => self.add_token(TokenType::Bang, None),
            b'=' if self.peek() == b'=' => {
                self.advance();
                self.add_token(TokenType::EqualEqual, None)
            },
            b'=' => self.add_token(TokenType::Equal, None),
            b'>' if self.peek() == b'=' => {
                self.advance();
                self.add_token(TokenType::GreaterEqual, None)
            },
            b'>' => self.add_token(TokenType::Greater, None),
            b'<' if self.peek() == b'=' => {
                self.advance();
                self.add_token(TokenType::LessEqual, None)
            },
            b'<' => self.add_token(TokenType::Less, None),
            b'/' if self.peek() == b'/' => {
                while self.peek() != b'\n' && !self.is_at_end() {
                    self.advance();
                }
            },
            b'/' => self.add_token(TokenType::Slash, None),
            b' ' | b'\t' | b'\r' => {},
            b'\n' => {
                self.line += 1;
            },
            b'"' => self.string(),
            c if is_digit(c) => self.number(),
            c if is_alpha(c) => self.identifier(),
            c => {
                self.reporter.report(LoxError::scan(
                    self.line,
                    format!("unexpected character: {:?}", c as char),
                ));
            },
        }
    }

    fn advance(&mut self) -> u8 {
        self.current += 1;
        self.source.as_bytes()[self.current - 1]
    }

    fn add_token<V>(&mut self, ty: TokenType, literal: V)
    where
        V: Into<Option<Value>>,
    {
        let text = self.source[self.start..self.current].into();
        let literal = literal.into().unwrap_or(Value::Nil);
        self.tokens.push(Token::new(ty, text, literal, self.line))
    }

    fn peek(&self) -> u8 {
        if self.is_at_end() {
            b'\0'
        } else {
            self.source.as_bytes()[self.current]
        }
    }

    fn peek_next(&self) -> u8 {
        if self.current + 1 >= self.source.len() {
            b'\0'
        } else {
            self.source.as_bytes()[self.current + 1]
        }
    }

    fn string(&mut self) {
        while self.peek() != b'"' && !self.is_at_end() {
            if self.peek() == b'\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            self.reporter
                .report(LoxError::scan(self.line, "unterminated string"));
            return;
        }

        self.advance();

        let value = self.source[self.start + 1..self.current - 1].into();

        self.add_token(TokenType::String, Value::String(value));
    }

    fn number(&mut self) {
        while is_digit(self.peek()) {
            self.advance();
        }

        if self.peek() == b'.' && is_digit(self.peek_next()) {
            self.advance();

            while is_digit(self.peek()) {
                self.advance();
            }
        }

        self.add_token(
            TokenType::Number,
            Value::Number(
                f64::from_str(
                    self.source.get(self.start..self.current).unwrap(),
                )
                .unwrap(),
            ),
        );
    }

    fn identifier(&mut self) {
        while is_alpha_numeric(self.peek()) {
            self.advance();
        }

        let text = self.source.get(self.start..self.current).unwrap();

        let ty =
            RESERVED_WORDS.get(text).cloned().unwrap_or(TokenType::Identifier);

        self.add_token(ty, None);
    }
}

fn is_digit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

fn is_alpha(c: u8) -> bool {
    (c >= b'a' && c <= b'z') || (c >= b'A' && c <= b'Z') || c == b'_'
}

fn is_alpha_numeric(c: u8) -> bool {
    is_digit(c) || is_alpha(c)
}
