use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::str;

use crate::{
  interpreter::Interpreter,
  parser::{Parser, ParserOutcome, state::ParserOptions},
  resolver::{Resolver, error::ErrorType},
};

fn handle_parser_outcome(
  // src: &str,
  (stmts, errors): &ParserOutcome,
  interpreter: &mut Interpreter,
) -> bool {
  // parse errors
  if !errors.is_empty() {
    for error in errors {
      eprintln!("{}", error);
    }
    return false;
  }

  // resolver errors
  let resolver = Resolver::new(interpreter);
  let (ok, errors) = resolver.resolve(stmts);
  if !ok {
    let mut has_errors = false;
    for error in errors {
      eprintln!("{}; at position {}", error.message, error.span);
      if let ErrorType::Error = error.kind {
        has_errors = true;
      };
    }
    if has_errors { return false;}
  }

  // interpreter
  if let Err(error) = interpreter.interpret(stmts) {
    eprintln!("{}", error);
    // print_span_window(writer, src, error.primary_span());
    return false;
  }
  true
}

pub fn run_file(file: impl AsRef<Path>) -> io::Result<bool> {
  let src = &fs::read_to_string(file)?;
  let mut interpreter = Interpreter::new();

  Ok(run(src, &mut interpreter, ParserOptions {
    repl_mode: false,
    display_ast: true,
  }))
}

/// Process Lox source code
fn run(src: &str, interpreter: &mut Interpreter, options: ParserOptions) -> bool {
  let mut parser = Parser::new(src);
  parser.options = options;

  let outcome = parser.parse();

  handle_parser_outcome(&outcome, interpreter)
}

/// REPL mode
pub fn run_repl() {
  println!("Entering interactive mode...");
  let mut interpreter = Interpreter::new();

  let options = ParserOptions {
    repl_mode: true,
    display_ast: false,
  };

  loop {
    let mut line = String::new();
    print!("> ");
    io::stdout().flush().unwrap();

    io::stdin()
      .read_line(&mut line)
      .expect("Failed to read line");

    if !run(&line, &mut interpreter, options.clone()) {
      continue;
    };
  }
}
