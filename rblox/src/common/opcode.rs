use std::fmt::Display;

use crate::common::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
  Constant(Value),
  Return,
}

impl Display for OpCode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    const PAD: usize = 15;
    use OpCode::*;
    match self {
      Constant(val) => write!(f, "{:PAD$}{val}", "OP_CONST"),
      Return => write!(f, "OP_RETURN"),

    }
  }
}
