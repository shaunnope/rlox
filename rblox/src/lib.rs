mod common;
mod vm;
mod compiler;
mod gc;
mod user;

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
    return Err("Usage: rlox [script]");
  }

  if let Err(err) = user::run_file(&file_path) {
    eprintln!("{}", err);
    return Err("Could not run file")
  };

  Ok(())
}
