use std::fs;
use std::io::{self, Write};
use std::str;
// use std::env;

use error::Error;


#[cfg(test)]
mod tests;

mod scanner;
mod error;

fn run_file(path: &str) -> Result<(), Error> {
  let bytes = fs::read(path)?;
  run(str::from_utf8(&bytes)?)?;

  Ok(())
}

fn run(source: &str) -> Result<(), Error> {
  // process source code
  for token in scanner::scan_tokens(source) {
    println!("{token:?}");
  };

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

pub fn error(line: i32, message: &str) {
  report(line, "", message);
}



fn report(line: i32, at: &str, message: &str) {
  eprintln!("[line {line}] Error {at}: {message}");
  // error::set();
}