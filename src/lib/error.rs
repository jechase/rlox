use failure::{
    Error,
    Fail,
};

use display_derive::Display;

use std::fmt::{
    self,
    Display,
};

#[derive(Default, Debug)]
pub struct Reporter {
    errors: Vec<Error>,
}

#[derive(Fail, Debug)]
pub struct Errors(Vec<Error>);

impl Display for Errors {
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

impl Reporter {
    pub fn finish(self) -> Result<(), Errors> {
        if self.errors.len() == 0 {
            Ok(())
        } else {
            Err(Errors(self.errors))
        }
    }

    pub fn report<E>(&mut self, error: E)
    where
        E: Into<Error>,
    {
        self.errors.push(error.into());
    }
}

#[derive(Fail, Debug, Display)]
#[display(fmt = "[line {}] Error{}: {}", line, loc, msg)]
pub struct LoxError {
    line: usize,
    loc:  String,
    msg:  String,
}

impl LoxError {
    pub fn scan<S>(line: usize, msg: S) -> LoxError
    where
        S: Into<String>,
    {
        LoxError {
            line,
            loc: "".into(),
            msg: msg.into(),
        }
    }
}
