use std::fmt::Debug;

use crate::{
  common::Value,
  compiler::scanner::token::TokenType
};

#[derive(Clone, PartialEq)]
pub enum Ins {
  Constant(Value),
  Add,
  Subtract,
  Multiply,
  Divide,
  Negate,
  Return,
}

impl Debug for Ins {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    const PAD: usize = 15;
    use Ins::*;
    match self {
      Constant(val) => write!(f, "{:PAD$}{val:?}", "OP_CONST"),
      Add => write!(f, "OP_ADD"),
      Subtract => write!(f, "OP_SUB"),
      Multiply => write!(f, "OP_MUL"),
      Divide => write!(f, "OP_DIV"),
      Negate => write!(f, "OP_NEG"),
      Return => write!(f, "OP_RETURN"),
    }
  }
}

impl From<f64> for Ins {
  fn from(value: f64) -> Self {
    Self::Constant(Value::from(value))
  }
}
