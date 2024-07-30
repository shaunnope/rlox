#![allow(dead_code)]

use std::error;
use std::fmt;

use std::sync::atomic::{AtomicBool, Ordering};

static HAD_ERROR: AtomicBool = AtomicBool::new(false);
static HAD_RUNTIME_ERROR: AtomicBool = AtomicBool::new(false);


pub type Error = Box<dyn error::Error>;

#[derive(Debug, Clone)]
pub enum Type {
  Parse,
  Runtime
}

fn set_flag(flag: &Type) {
  match flag {
    Type::Parse => HAD_ERROR.store(true, Ordering::Relaxed),
    Type::Runtime => HAD_RUNTIME_ERROR.store(true, Ordering::Relaxed),
  }
  
}

#[derive(Debug, Clone)]
pub struct PartialErr {
  pub err: Type,
  pub message: String
}

impl PartialErr {
  pub fn new(err: Type, message: &str) -> Self {
    Self { err, message: message.to_string() }
  }
}

#[derive(Debug, Clone)]
pub struct LoxError {
  pub err: Type,
  pub line: i32,
  pub pos: String,
  pub message: String,

}

impl LoxError {
  pub fn new(err: Type, line: i32, pos: &str, message: &str) -> Self {
    set_flag(&err);

    Self { err, line, pos: pos.to_string(), message: message.to_string() }
  }

  pub fn from(part: PartialErr, line: i32, pos: &str) -> Self {
    set_flag(&part.err);

    Self { err: part.err, line, pos: pos.to_string(), message: part.message }
  }

  pub fn report(err: Type, line: i32, pos: &str, message: &str) {
    eprintln!("{}", Self::new(err, line, pos, message));
  }

  pub fn display(&self) {
    eprintln!("{self}");
  }
}

impl fmt::Display for LoxError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "[line {}] Error{}: {}", self.line, self.pos, self.message)
  }
}

impl error::Error for LoxError {}
