use std::env;
use std::process;

// use rlox::ast::example;

fn main() {
    // example();

    let _ = rlox::parse_args(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {err}");
        process::exit(1);
    });

    // println!("Searching for {}", config.query);
    // println!("In file {}", config.file_path);

    // if let Err(e) = rlox::run(config) {
    //     eprintln!("Application error: {e}");
    //     process::exit(1);
    // }
}
