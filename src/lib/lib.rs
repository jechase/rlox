#![recursion_limit = "512"]

mod ast;
mod error;
mod interpreter;
mod parser;
mod print_ast;
mod scanner;
mod token;
mod value;

#[allow(unused_imports)]
use self::{
    ast::*,
    error::*,
    interpreter::*,
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
    let mut scanner_reporter = Reporter::new();
    let mut parser_reporter = Reporter::new();

    let scanner = scanner_reporter.filter(Scanner::new(source));

    let parser = parser_reporter.filter(Parser::new(scanner));

    let exprs: Vec<_> = parser.collect();

    scanner_reporter.join(parser_reporter);
    scanner_reporter.finish()?;

    let mut interpreter = Interpreter;

    for expr in exprs {
        println!("{}", interpreter.interpret(&expr)?);
    }

    Ok(())
}
