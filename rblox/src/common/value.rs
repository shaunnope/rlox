use std::{
  fmt::{Debug, Display},
  ops::{Neg, Not},
  rc::Rc
};

use crate::common::data::LoxObject;

#[derive(Clone, PartialEq)]
pub enum Value {
  Boolean(bool),
  Nil,
  Number(f64),
  Object(Rc<LoxObject>)
}

impl Value {
  /// Returns the canonical type name.
  pub fn type_name(&self) -> &'static str {
    use Value::*;
    match self {
      Boolean(_) => "boolean",
      Number(_) => "number",
      Nil => "nil",
      Object(obj) => obj.type_name()
      // Unset => "<unset>",
    }
  }

  /// Converts a `LoxValue` to a Rust bool
  pub fn truth(&self) -> bool {
    match self {
      Self::Boolean(val) => *val,
      Self::Nil => false,
      _ => true
    }
  }

  /// Checks if two `LoxValue`s are equal. No type coercion is performed so both types must be equal.
  pub fn equals(&self, other: &Self) -> bool {
    use Value::*;
    match (self, other) {
      (Boolean(a), Boolean(b)) => a == b,
      (Number(a), Number(b)) => a == b,
      (Nil, Nil) => true,
      (Object(a), Object(b)) => a == b,
      _ => false,
    }
  }

  /// Creates a copy if literal, else clones the LoxObject pointer
  pub fn copy(&self) -> Self {
    use Value::*;
    match self {
      Boolean(val) => Self::Boolean(*val),
      Number(val) => Self::Number(*val),
      Nil => Self::Nil,
      Object(obj) => Self::Object(obj.clone())
    }
  }
}

impl Debug for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Value::*;
    match self {
      Boolean(b) => write!(f, "{b}"),
      Nil => write!(f, "nil"),
      Number(n) => {
        if n.floor() == *n {
          write!(f, "{n:.0}")
        } else {
          write!(f, "{n}")
        }
      },
      Object(obj) => write!(f, "{obj:?}")
    }
  }
}

impl Display for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Value::*;
    match self {
      Object(obj) => write!(f, "{obj}"),
      other => write!(f, "{other:?}")
    }
  }
}

impl Neg for Value {
  type Output = Self;
  fn neg(self) -> Self::Output {
    use Value::*;
    match self {
      Number(n) => Number(-n),
      _ => unreachable!("Illegal use of `-` on non-numeric value")
    }
  }
}

impl Not for Value {
  type Output = bool;

  fn not(self) -> Self::Output {
    match self {
      Value::Nil => false,
      val => !val.truth()
    }
  }
}

impl From<f64> for Value {
  fn from(value: f64) -> Self {
    Self::Number(value)
  }
}
