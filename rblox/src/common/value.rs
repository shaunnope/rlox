use std::{
  fmt::Debug,
  ops::{Neg, Not}
};

#[derive(Clone, PartialEq)]
pub enum Value {
  Boolean(bool),
  Nil,
  Number(f64),
}

impl Value {
  /// Returns the canonical type name.
  pub fn type_name(&self) -> &'static str {
    use Value::*;
    match self {
      Boolean(_) => "boolean",
      Number(_) => "number",
      // String(_) => "string",
      Nil => "nil",
      // Function(_) => "<func>",
      // Class(_) => "<class>",
      // Object(_) => "<instance>",
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
      // (String(a), String(b)) => a == b,
      (Nil, Nil) => true,
      _ => false,
    }
  }

}

impl Debug for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Value::*;
    match self {
      Boolean(b) => b.fmt(f),
      Nil => write!(f, "nil"),
      Number(n) => {
        if n.floor() == *n {
          write!(f, "{:.0}", n)
        } else {
          write!(f, "{}", n)
        }
      },
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