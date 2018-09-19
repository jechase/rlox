#![feature(nll)]
#![recursion_limit = "1024"]
#![feature(transpose_result)]

mod ast;
mod callable;
mod class;
mod environment;
mod error;
mod instance;
mod interpreter;
mod parser;
mod print_ast;
mod resolver;
mod run;
mod scanner;
mod token;
mod value;

#[allow(unused_imports)]
use self::{
    ast::*,
    callable::*,
    class::*,
    environment::*,
    error::*,
    instance::*,
    interpreter::*,
    parser::*,
    print_ast::*,
    resolver::*,
    scanner::*,
    token::*,
    value::*,
};

pub use crate::run::*;

use tendril::StrTendril;

type LoxStr = StrTendril;
