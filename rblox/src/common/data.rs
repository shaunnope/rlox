use std::{fmt::Display, mem};


#[derive(Debug, Clone, PartialEq)]
pub enum LoxObject {
  String(String),
}

impl LoxObject {
  /// Returns the canonical type name.
  pub fn type_name(&self) -> &'static str {
    use LoxObject::*;
    match self {
      String(_) => "string",
      // Function(_) => "<func>",
      // Class(_) => "<class>",
      // Object(_) => "<instance>",
      // Unset => "<unset>",
    }
  }

  pub fn is_type(&self, other: LoxObject) -> bool {
    mem::discriminant(self) == mem::discriminant(&other)
  }
}

impl Display for LoxObject {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use LoxObject::*;
    match self {
      String(s) => write!(f, "{s}"),
    }
  }
}