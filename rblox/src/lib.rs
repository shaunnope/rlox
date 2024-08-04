mod common;

use std::str;

use crate::common::{Chunk, OpCode};


pub fn parse_args(mut args: impl Iterator<Item = String>) -> Result<(), &'static str> {
  args.next();

  run();

  // let file_path = match args.next() {
  //   Some(arg) => arg,
  //   None => {
  //     // user::run_repl();
  //     return Ok(());
  //   }
  // };

  // // don't accept extra arguments
  // if let Some(_) = args.next() {
  //   return Err("Usage rlox [script]");
  // }

  // if let Err(err) = user::run_file(&file_path) {
  //   eprintln!("{}", err);
  //   return Err("Could not run file")
  // };

  Ok(())
}


fn run() {
  let mut chunk = Chunk::new("test chunk");
  chunk.write(OpCode::Constant(common::Value::Number(1.2)), 1);
  chunk.write(OpCode::Constant(common::Value::Number(2.0)), 2);
  chunk.write(OpCode::Return, 2);
  println!("{}", chunk);
}