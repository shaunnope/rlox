use std::{fs, error::Error, path::Path};

use super::*;

const TEST_DIR: &str = "../tests/";


#[test]  
fn sequence() -> Result<(), Box<dyn Error>> {
  let path = Path::new(TEST_DIR)
    .join("sequence")
    .join("assignment.lox");
  println!("\n{:?}", path);
  let src = &fs::read_to_string(path)?;

  let mut vm = VM::new();
  
  if let Err(err) = vm.run(src) {
    eprintln!("{err:?}")
  };
  
  Ok(())
}