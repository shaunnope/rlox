use std::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ErrorLevel {
  _Debug,
  _Warning,
  Error
}

#[derive(Debug)]
pub enum ErrorType {
  Error,
  CompileError,
  _RuntimeError,
}

pub trait Error: Display + Debug {
  fn get_level(&self) -> ErrorLevel;
  fn get_type(&self) -> ErrorType;
}

pub type LoxResult<T> = Result<(), T>;

// pub enum LoxError {
//   Error(Box<dyn Error>),
//   CompileError(Box<dyn Error>),
//   RuntimeError(Box<dyn Error>),
// }


// impl LoxError {
//   pub fn dummy() -> Self {
//     Self::Error(Box::new(DummyError {}))
//   }

//   pub fn get_level(&self) -> ErrorLevel {
//     use LoxError::*;
//     match self {
//       Error(err) | CompileError(err) | RuntimeError(err) => err.get_level()
//     }
//   }
// }

#[derive(Debug)]
pub struct DummyError {}

impl Display for DummyError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

impl Error for DummyError {
  fn get_level(&self) -> ErrorLevel {
    ErrorLevel::Error
  }
  fn get_type(&self) -> ErrorType {
    ErrorType::Error
  }
}