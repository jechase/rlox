#![recursion_limit = "512"]

mod ast;
mod error;
mod parser;
mod print_ast;
mod scanner;
mod token;
mod value;

#[allow(unused_imports)]
use self::{
    ast::*,
    error::*,
    parser::*,
    print_ast::*,
    scanner::*,
    token::*,
    value::*,
};

use failure::Error;

type LoxStr = tendril::StrTendril;

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
    let mut reporter = Reporter::default();
    let scanner = Scanner::new(source).filter_map(|res| match res {
        Ok(t) => Some(t),
        Err(e) => {
            reporter.report(e);
            None
        },
    });
    let mut parser = Parser::new(scanner);

    match parser.parse() {
        Ok(expr) => println!("{}", AstPrinter.visit(&expr)),
        Err(e) => reporter.report(e),
    }

    reporter.finish().map_err(From::from)
}
