use std::{fmt::Debug, rc::Rc};

use crate::common::{Value, data::LoxObject};

#[derive(Clone, PartialEq)]
pub enum Ins {
  // literals
  Constant(Value), True, False, Nil,

  // arithmetic
  Add, Subtract, Multiply, Divide,
  Negate,

  Not, // Or, And, 
  Equal, Greater, Less,

  DefGlobal(String),
  GetGlobal(String),
  SetGlobal(String),

  Print, Pop,
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
      // Or => write!(f, "OP_OR"),
      // And => write!(f, "OP_AND"),
      Equal => write!(f, "OP_EQUAL"),
      Greater => write!(f, "OP_GREATER"),
      Less => write!(f, "OP_LESS"),

      DefGlobal(var) => write!(f, "{:PAD$}{var}", "OP_DEF_GLOB"),
      GetGlobal(var) => write!(f, "{:PAD$}{var}", "OP_GET_GLOB"),
      SetGlobal(var) => write!(f, "{:PAD$}{var}", "OP_SET_GLOB"),

      Print => write!(f, "OP_PRINT"),
      Pop => write!(f, "OP_POP"),
      Return => write!(f, "OP_RETURN"),
    }
  }
}

impl From<f64> for Ins {
  fn from(value: f64) -> Self {
    Self::Constant(Value::from(value))
  }
}

impl From<LoxObject> for Ins {
  fn from(value: LoxObject) -> Self {
    Self::Constant(Value::Object(Rc::new(value)))
  }
}
