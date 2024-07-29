#![allow(dead_code)]

use std::error;
use std::fmt;

use std::sync::atomic::{AtomicBool, Ordering};

static HAD_ERROR: AtomicBool = AtomicBool::new(false);

pub type Error = Box<dyn error::Error>;
pub trait ErrorTrait: error::Error {
  fn report(&self) {
    eprintln!("{self}");
  }
}

#[derive(Debug, Clone)]
pub struct ParseError {
  pub line: i32,
  pub pos: String,
  pub message: String,

}

impl ParseError {
  pub fn new(line: i32, pos: &str, message: &str) -> Self {
    HAD_ERROR.store(true, Ordering::Relaxed);
    ParseError { line, pos: pos.to_string(), message: message.to_string() }
  }

  pub fn report(line: i32, pos: &str, message: &str) {
    eprintln!("{}", Self::new(line, pos, message));
  }

  pub fn display(&self) {
    eprintln!("{self}");
  }
}

impl fmt::Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "[line {}] Error{}: {}", self.line, self.pos, self.message)
  }
}

impl error::Error for ParseError {}
impl ErrorTrait for ParseError {}