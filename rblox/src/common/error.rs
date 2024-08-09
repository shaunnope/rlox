use std::{
  error::Error as StdError,
  fmt::Debug
};

use super::Span;

#[derive(Clone, PartialEq, PartialOrd)]
pub enum ErrorLevel {
  _Info,
  Warning,
  Error
}

impl Debug for ErrorLevel {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use ErrorLevel::*;
    match self {
      _Info => write!(f, "INFO"),
      Warning => write!(f, "WARNING"),
      Error => write!(f, "ERROR"),
    }
  }
}

pub enum ErrorType {
  _Error,
  CompileError,
  RuntimeError,
}

impl Debug for ErrorType {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      use ErrorType::*;
      match self {
        _Error => write!(f, "Error"),
        CompileError => write!(f, "Compile Error"),
        RuntimeError => write!(f, "Runtime Error"),
      }
  }
}

pub trait LoxError: StdError {
  fn get_level(&self) -> ErrorLevel;
  fn get_type(&self) -> ErrorType;
  fn get_span(&self) -> Span;

  fn report(&self) {
    eprintln!("[{:?} line {}] {:?}: {}", self.get_level(), self.get_span().2, self.get_type(), self)
  }
}

pub type LoxResult<T> = Result<(), T>;
