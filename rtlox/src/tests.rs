// use super::*;

use std::{fs, error::Error, path::Path};

use rtlox::user::run_file;

const TEST_DIR: &str = "../tests/";


#[test]  
fn recursion() -> Result<(), Box<dyn Error>> {
  let path = Path::new(TEST_DIR).join("function").join("recursion.lox");
  println!("\n{:?}", path);
  run_file(path)?;
  
  Ok(())
}


macro_rules! sanity_checks {
  ($($name:ident: $value:expr,)*) => {
  $(
      #[test]
      fn $name() -> Result<(), Box<dyn Error>> {
        let test_dir = Path::new(TEST_DIR).join($value);
        for fname in fs::read_dir(test_dir)? {
          let path = fname?.path();
          println!("\n{:?}", path);
          run_file(path)?;
        };

        Ok(())
      }
  )*
  }
}

sanity_checks! {
  assignment: "assignment",
  // benchmark: "benchmark",
  block: "block",
  bool: "bool",
  call: "call",
  class: "class",
  closure: "closure",
  comments: "comments",
  constructor: "constructor",
  function: "function",
}