#![feature(nll)]
#![recursion_limit = "1024"]

mod ast;
mod callable;
mod error;
mod interpreter;
mod parser;
mod print_ast;
mod scanner;
mod scope;
mod token;
mod value;

#[allow(unused_imports)]
use self::{
    ast::*,
    callable::*,
    error::*,
    interpreter::*,
    parser::*,
    print_ast::*,
    scanner::*,
    scope::*,
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

use tendril::StrTendril;

type LoxStr = StrTendril;

pub fn run_file<P>(path: P) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let contents = read_to_string(path)?;
    let mut interpreter = Interpreter::default();
    run(&mut interpreter, &contents)
}

pub fn run_prompt() -> Result<(), Error> {
    let mut stdin = io::BufReader::new(io::stdin());
    let mut stdout = io::stdout();

    let mut interpreter = Interpreter::default();

    let mut line = String::new();
    loop {
        write!(stdout, "> ")?;
        stdout.flush()?;
        stdin.read_line(&mut line)?;
        if let Err(e) = run(&mut interpreter, &line) {
            println!("{}", e);
        }
        line.clear();
    }
}

pub fn run(interpreter: &mut Interpreter, source: &str) -> Result<(), Error> {
    let mut scanner_reporter = Reporter::new();
    let mut parser_reporter = Reporter::new();

    let scanner = scanner_reporter.filter(scan(source));

    let parser = parser_reporter.filter(Parser::new(scanner));

    let stmts: Vec<_> = parser.collect();

    scanner_reporter.join(parser_reporter);
    scanner_reporter.finish()?;

    for stmt in stmts {
        match stmt {
            Stmt::Expr(ref e) => println!("{}", interpreter.evaluate(e)?),
            _ => {
                interpreter.execute(&stmt)?;
            },
        }
    }

    Ok(())
}
