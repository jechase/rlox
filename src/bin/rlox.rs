use std::{
    env,
    process,
};

fn main() {
    let mut args = env::args();
    if args.len() > 2 {
        println!("Usage: rlox [script]");
        process::exit(64);
    } else if args.len() == 2 {
        rlox::run_file(args.nth(1).unwrap())
    } else {
        rlox::run_prompt()
    }
}
