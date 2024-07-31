use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::str;

use crate::interpreter::{self, Interpreter};
use crate::parser::{Parser, ParserOutcome};

// use error::Error;

fn handle_parser_outcome(
  // src: &str,
  (stmts, errors): &ParserOutcome,
  interpreter: &mut Interpreter,
) -> bool {
  // parse errors
  if !errors.is_empty() {
    for error in errors {
      eprintln!("{:?}", error);
    }
    return false;
  }

  // interpreter
  if let Err(error) = interpreter.interpret(stmts) {
    eprintln!("{:?}\n", error);
    // print_span_window(writer, src, error.primary_span());
    return false;
  }
  true
}

pub fn run_file(file: impl AsRef<Path>) -> io::Result<bool> {
  let src = &fs::read_to_string(file)?;
  let mut interpreter = Interpreter {};

  Ok(run(src, &mut interpreter))
}

fn run(src: &str, interpreter: &mut Interpreter) -> bool {
  // process source code
  let outcome = Parser::new(src).parse();

  handle_parser_outcome(&outcome, interpreter)
}

pub fn run_repl() {
  // REPL mode
  println!("Entering interactive mode...");
  let mut interpreter = Interpreter {};

  loop {
    let mut line = String::new();
    print!("> ");
    io::stdout().flush().unwrap();

    io::stdin()
      .read_line(&mut line)
      .expect("Failed to read line");

    if !run(&line, &mut interpreter) {
      continue;
    };
  }
}
