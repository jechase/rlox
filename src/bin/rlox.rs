use std::env;

fn main() {
    let mut args = env::args();
    let res = if args.len() > 2 {
        println!("Usage: rlox [script]");
        return;
    } else if args.len() == 2 {
        rlox::run_file(args.nth(1).unwrap())
    } else {
        rlox::run_prompt()
    };
    if let Err(err) = res {
        println!("Error: {}", err);
    }
}
