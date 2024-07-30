#[cfg(test)]
mod tests;

pub mod ast;
pub mod interpreter;
pub mod parser;
pub mod token;

mod error; // custom error type

pub mod data;
pub mod span;
pub mod user;

use std::str;

pub fn parse_args(mut args: impl Iterator<Item = String>) -> Result<(), &'static str> {
  args.next();

  let file_path = match args.next() {
    Some(arg) => arg,
    None => {
      user::run_repl();
      return Ok(());
    }
  };

  // don't accept extra arguments
  if let Some(_) = args.next() {
    return Err("Usage rlox [script]");
  }

  let _ = user::run_file(&file_path);

  Ok(())
}
