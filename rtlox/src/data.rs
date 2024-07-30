// mod eq;

use std::fmt::{self, Debug, Display};

/// A dynamic type representation of lox values.
///
/// None for lox's nil
// pub type Evaluation = Option<Box<dyn eq::DynEq>>;

#[derive(Clone)]
pub enum LoxValue {
  Boolean(bool),
  Number(f64),
  String(String),
  Nil,
}

impl LoxValue {
  /// Returns the canonical type name.
  pub fn type_name(&self) -> &'static str {
    use LoxValue::*;
    match self {
      Boolean(_) => "boolean",
      Number(_) => "number",
      String(_) => "string",
      Nil => "nil",
    }
  }

  /// Converts a `LoxValue` to a Rust bool
  pub fn truth(&self) -> bool {
    use LoxValue::*;
    match self {
      Boolean(inner) => *inner,
      Number(_) | String(_) => true,
      Nil => false,
    }
  }

  /// Checks if two `LoxValue`s are equal. No type coercion is performed so both types must be equal.
  pub fn equals(&self, other: &Self) -> bool {
    use LoxValue::*;
    match (self, other) {
      (Boolean(a), Boolean(b)) => a == b,
      (Number(a), Number(b)) => a == b,
      (String(a), String(b)) => a == b,
      (Nil, Nil) => true,
      _ => false,
    }
  }
}

impl Display for LoxValue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use LoxValue::*;
    match self {
      Boolean(boolean) => Display::fmt(boolean, f),
      Number(number) => {
        if number.floor() == *number {
          write!(f, "{:.0}", number)
        } else {
          Display::fmt(number, f)
        }
      }
      String(string) => f.write_str(string),
      Nil => f.write_str("nil"),
    }
  }
}

impl Debug for LoxValue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use LoxValue::*;
    match self {
      String(s) => write!(f, "\"{}\"", s),
      other => Display::fmt(other, f),
    }
  }
}
