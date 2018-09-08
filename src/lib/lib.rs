#[macro_use]
extern crate lazy_static;

mod scanner;
mod token;
mod value;

use self::{
    scanner::*,
    token::*,
    value::*,
};

use std::{
    fs::read_to_string,
    io::{
        self,
        BufRead,
        Write,
    },
    path::Path,
    process,
    sync::atomic::{
        AtomicBool,
        Ordering,
        ATOMIC_BOOL_INIT,
    },
};

static HAD_ERROR: AtomicBool = ATOMIC_BOOL_INIT;

pub fn run_file<P>(path: P)
where
    P: AsRef<Path>,
{
    let contents = read_to_string(path).unwrap();
    run(&contents);
    if had_error() {
        process::exit(64);
    }
}

pub fn run_prompt() {
    let mut stdin = io::BufReader::new(io::stdin());
    let mut stdout = io::stdout();

    let mut line = String::new();
    loop {
        write!(stdout, "> ").unwrap();
        stdout.flush().unwrap();
        stdin.read_line(&mut line).unwrap();
        run(&line);
        set_error(false);
        line.clear();
    }
}

pub fn run(source: &str) {
    let scanner = Scanner::new(source.into());
    let tokens = scanner.scan_tokens();

    for token in tokens {
        println!("{:?}", token);
    }
}

fn had_error() -> bool {
    HAD_ERROR.load(Ordering::Relaxed)
}

fn set_error(err: bool) {
    HAD_ERROR.store(err, Ordering::Relaxed)
}

fn error(line: usize, message: &str) {
    report(line, "", message);
}

fn report(line: usize, loc: &str, message: &str) {
    println!("[line {}] Error{}: {}", line, loc, message);
    HAD_ERROR.store(true, Ordering::Relaxed);
}
