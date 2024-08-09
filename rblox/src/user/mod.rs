use std::{
  fs,
  io::{self, Write},
  path::Path,
};

use crate::vm::VM;

pub fn run_file(file: impl AsRef<Path>) -> io::Result<bool> {
  let src = &fs::read_to_string(file)?;
  let mut vm = VM::new();
  
  Ok(run(src, &mut vm))
}

/// Process Lox source code
fn run(src: &str, vm: &mut VM) -> bool {
  match vm.run(src) {
    Err(_) => false,
    Ok(_) => true
  }
}

/// REPL mode
pub fn run_repl() {
  println!("Entering interactive mode...");
  let mut vm = VM::new();

  loop {
    let mut line = String::new();
    print!("> ");
    io::stdout().flush().unwrap();

    io::stdin()
      .read_line(&mut line)
      .expect("Failed to read line");

    if !run(&line, &mut vm) {
      continue;
    };
  }
}
