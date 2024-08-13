use std::{fmt::Debug, rc::Rc};

use crate::common::{Value, data::LoxObject};

#[derive(Clone, PartialEq)]
pub enum Ins {
  // literals
  Constant(Value), True, False, Nil,

  // arithmetic
  Add, Subtract, Multiply, Divide,
  Negate,

  Not,
  Equal, Greater, Less,

  DefGlobal(String),
  GetGlobal(String),
  SetGlobal(String),

  GetLocal(usize),
  SetLocal(usize),

  GetUpval(usize),
  SetUpval(usize),

  Call(usize),
  Closure(usize, Rc<Vec<(bool, usize)>>),

  Jump(isize),
  JumpIfFalse(isize),
  // Loop(usize),

  Print, Pop, PopN(usize),
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
      Equal => write!(f, "OP_EQUAL"),
      Greater => write!(f, "OP_GREATER"),
      Less => write!(f, "OP_LESS"),

      DefGlobal(var) => write!(f, "{:PAD$}{var}", "OP_DEF_GLOB"),
      GetGlobal(var) => write!(f, "{:PAD$}{var}", "OP_GET_GLOB"),
      SetGlobal(var) => write!(f, "{:PAD$}{var}", "OP_SET_GLOB"),

      GetLocal(var) => write!(f, "{:PAD$}{var}", "OP_GET_LOC"),
      SetLocal(var) => write!(f, "{:PAD$}{var}", "OP_SET_LOC"),

      GetUpval(var) => write!(f, "{:PAD$}{var}", "OP_GET_UPV"),
      SetUpval(var) => write!(f, "{:PAD$}{var}", "OP_SET_UPV"),

      Call(args) => write!(f, "{:PAD$}{args}", "OP_CALL"),
      Closure(n, upvals) => {
        write!(f, "{:PAD$}{n}  ", "OP_CLOSURE")?;
        // TODO: print upvalue
        for val in upvals.iter() {
          write!(f, "{}{} ", if val.0 {">"} else {"^"}, val.1 )?;
        }
        Ok(())
      },

      Jump(n) => write!(f, "{:PAD$}{n}", "OP_JMP"),
      JumpIfFalse(n) => write!(f, "{:PAD$}{n}", "OP_JMPF"),

      Print => write!(f, "OP_PRINT"),
      Pop => write!(f, "OP_POP"),
      PopN(n) => write!(f, "{:PAD$}{n}", "OP_POPN"),
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
