#[macro_use]
extern crate lazy_static;

mod error;
mod scanner;
mod token;
mod value;

use self::{
    error::*,
    scanner::*,
    token::*,
    value::*,
};

use failure::Error;

use std::{
    fs::read_to_string,
    io::{
        self,
        BufRead,
        Write,
    },
    path::Path,
};

pub fn run_file<P>(path: P) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let contents = read_to_string(path)?;
    run(&contents)
}

pub fn run_prompt() -> Result<(), Error> {
    let mut stdin = io::BufReader::new(io::stdin());
    let mut stdout = io::stdout();

    let mut line = String::new();
    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;
        stdin.read_line(&mut line)?;
        if let Err(e) = run(&line) {
            println!("{}", e);
        }
        line.clear();
    }
}

pub fn run(source: &str) -> Result<(), Error> {
    let scanner = Scanner::new(source.into());
    let tokens = scanner.scan_tokens()?;

    for token in tokens {
        println!("{:?}", token);
    }

    Ok(())
}
