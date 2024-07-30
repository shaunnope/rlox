use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::str;

use crate::parser::{Parser, ParserOutcome};
use crate::interpreter::{self, Interpreter};

// use error::Error;

fn handle_parser_outcome(
  // src: &str,
  (stmts, errors): &ParserOutcome,
  // interpreter: &mut Interpreter,
) -> bool {
  // parse errors
  if !errors.is_empty() {
    for error in errors {
      eprintln!("{}\n", error);
    }
    return false;
  }

  let mut interpreter = Interpreter{};

  // interpreter
  if let Err(error) = interpreter.interpret(stmts) {
    eprintln!("{}\n", error);
    // print_span_window(writer, src, error.primary_span());
    return false;
  }
  true
}

pub fn run_file(file: impl AsRef<Path>) -> io::Result<bool> {
  let src = &fs::read_to_string(file)?;
  let _ = run(src);
  // let outcome = Parser::new(src).parse();
  // let status = handle_parser_outcome(
  //   src,
  //   &outcome,
  //   interpreter.unwrap_or(&mut Interpreter::new()),
  // );
  // Ok(status)
  Ok(true)
}

fn run(src: &str) -> Result<(), &'static str> {
  // process source code
  let outcome = Parser::new(src).parse();
  handle_parser_outcome(&outcome);

  // let tokens = scanner::scan_tokens(source)?;
  // if let Some(expr) = parser::parse(tokens) {
  //   println!("{}", expr);
  // }

  Ok(())
}

pub fn run_repl() {
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
