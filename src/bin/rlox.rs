use std::env;

use failure::{
    format_err,
    Error,
};

fn main() -> Result<(), Error> {
    let mut args = env::args();
    if args.len() > 2 {
        return Err(format_err!("Usage: rlox [script]"));
    } else if args.len() == 2 {
        rlox::run_file(args.nth(1).unwrap())
    } else {
        rlox::run_prompt()
    }
}
