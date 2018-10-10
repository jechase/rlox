use failure::Fail;

use display_derive::Display;

use std::fmt::{
    self,
    Display,
};

use crate::*;

#[derive(Debug)]
pub struct Reporter<E> {
    errors: Vec<E>,
}

impl<E> Default for Reporter<E>
where
    E: Fail,
{
    fn default() -> Self {
        Reporter::new()
    }
}

#[derive(Fail, Debug)]
pub struct Errors<E: Fail>(Vec<E>);

impl<E> Display for Errors<E>
where
    E: Fail,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (idx, err) in self.0.iter().enumerate() {
            if idx == self.0.len() - 1 {
                write!(f, "{}", err)?;
            } else {
                writeln!(f, "{}", err)?;
            }
        }
        Ok(())
    }
}

impl<E> Reporter<E>
where
    E: Fail,
{
    pub fn new() -> Reporter<E> {
        Reporter {
            errors: vec![],
        }
    }

    pub fn finish(self) -> Result<(), Errors<E>> {
        if self.errors.len() == 0 {
            Ok(())
        } else {
            Err(Errors(self.errors))
        }
    }

    pub fn report(&mut self, error: E) {
        self.errors.push(error);
    }
}

#[derive(Fail, Debug, Display)]
pub enum LoxError {
    #[display(fmt = "[line {}] Error: {}", _0, _1)]
    Scan(usize, String),
    #[display(fmt = "[line {}] Error{}: {}", _0, _1, _2)]
    Parse(usize, String, String),
}

impl LoxError {
    pub fn scan<S>(line: usize, msg: S) -> LoxError
    where
        S: Into<String>,
    {
        LoxError::Scan(line, msg.into())
    }

    pub fn parse<S>(token: &Token, msg: S) -> LoxError
    where
        S: Into<String>,
    {
        let loc = if token.ty == TokenType::Eof {
            " at end".into()
        } else {
            format!(" at {:?}", token.lexeme)
        };
        LoxError::Parse(token.line, loc, msg.into())
    }
}
