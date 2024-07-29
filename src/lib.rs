#[cfg(test)]
mod tests;

pub mod token;
pub mod scanner;
pub mod ast;
pub mod parser;


mod error; // custom error type


use std::fs;
use std::io::{self, Write};
use std::str;

use error::Error;



fn run_file(path: &str) -> Result<(), Error> {
  let bytes = fs::read(path)?;
  run(str::from_utf8(&bytes)?)?;

  Ok(())
}

fn run(source: &str) -> Result<(), Error> {
  // process source code
  let tokens = scanner::scan_tokens(source)?;
  if let Some(expr) = parser::parse(tokens) {
    println!("{}", expr);
  }

  Ok(())
}

fn run_prompt() {
  // REPL mode
  println!("Entering interactive mode...");
  loop {
    let mut line = String::new();
    print!("> ");
    io::stdout().flush().unwrap();

    io::stdin()
      .read_line(&mut line)
      .expect("Failed to read line");

    if let Err(_) = run(&line) {
      continue;
    };
  }
}

pub fn parse_args(
  mut args: impl Iterator<Item = String>,
) -> Result<(), &'static str> {
  args.next();

  let file_path = match args.next() {
    Some(arg) => arg,
    None => {
      run_prompt();
      return Ok(())
    },
  };

  // don't accept extra arguments
  if let Some(_) = args.next() {
    return Err("Usage rlox [script]")
  }

  let _ = run_file(&file_path);

  Ok(())
}
