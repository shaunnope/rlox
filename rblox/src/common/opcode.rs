use std::fmt::Debug;

use crate::common::Value;

#[derive(Clone, PartialEq)]
pub enum Ins {
  // literals
  Constant(Value), True, False, Nil,

  // arithmetic
  Add, Subtract, Multiply, Divide,
  Negate,

  Not, Or, And, 
  Equal, Greater, Less,
  Return,
}

impl Debug for Ins {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    const PAD: usize = 15;
    use Ins::*;
    match self {
      Constant(val) => write!(f, "{:PAD$}{val:?}", "OP_CONST"),
      True => write!(f, "OP_TRUE"),
      False => write!(f, "OP_FALSE"),
      Nil => write!(f, "OP_NIL"),

      Add => write!(f, "OP_ADD"),
      Subtract => write!(f, "OP_SUB"),
      Multiply => write!(f, "OP_MUL"),
      Divide => write!(f, "OP_DIV"),
      Negate => write!(f, "OP_NEG"),

      Not => write!(f, "OP_NOT"),
      Or => write!(f, "OP_OR"),
      And => write!(f, "OP_AND"),
      Equal => write!(f, "OP_EQUAL"),
      Greater => write!(f, "OP_GREATER"),
      Less => write!(f, "OP_LESS"),

      Return => write!(f, "OP_RETURN"),
    }
  }
}

impl From<f64> for Ins {
  fn from(value: f64) -> Self {
    Self::Constant(Value::from(value))
  }
}
