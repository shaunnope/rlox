use std::error;
use std::fmt;

// use std::sync::atomic::{AtomicBool, Ordering};

pub type Error = Box<dyn error::Error + 'static>;

#[derive(Debug)]
pub struct SyntaxError {
    message: String,
}

impl SyntaxError {
    pub fn new(message: String) -> Self {
        SyntaxError { message }
    }
}

impl fmt::Display for SyntaxError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl error::Error for SyntaxError {}