// mod eq;

use std::fmt::{self, Debug, Display};
use std::sync::atomic::{self, AtomicUsize};

use crate::span::Span;
use crate::token::{Token, TokenType};

#[derive(Clone)]
pub enum LoxValue {
  Boolean(bool),
  Number(f64),
  String(String),
  Nil,
  Unset,
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
      Unset => "<unset>",
    }
  }

  /// Converts a `LoxValue` to a Rust bool
  pub fn truth(&self) -> bool {
    use LoxValue::*;
    match self {
      Boolean(inner) => *inner,
      Number(_) | String(_) => true,
      Nil => false,
      Unset => unreachable!("Invalid access of unset variable."),
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
      Unset => f.write_str("<unset>"),
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

#[derive(Debug, Clone)]
pub struct LoxIdent {
  pub id: LoxIdentId,
  pub name: String,
  pub span: Span,
}

// global state:
static LOX_IDENT_ID_SEQ: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LoxIdentId(usize);

impl LoxIdentId {
  pub fn new() -> Self {
    LoxIdentId(LOX_IDENT_ID_SEQ.fetch_add(1, atomic::Ordering::SeqCst))
  }
}

impl LoxIdent {
  pub fn new(span: Span, name: impl Into<String>) -> Self {
    LoxIdent {
      id: LoxIdentId::new(),
      name: name.into(),
      span,
    }
  }
}

impl From<Token> for LoxIdent {
  fn from(Token { kind, span }: Token) -> Self {
    match kind {
      TokenType::Identifier(name) => LoxIdent::new(span, name),
      unexpected => unreachable!(
        "Invalid `Token` ({:?}) to `LoxIdent` conversion.",
        unexpected
      ),
    }
  }
}

impl AsRef<str> for LoxIdent {
  fn as_ref(&self) -> &str {
    &self.name
  }
}

impl From<LoxIdent> for String {
  fn from(ident: LoxIdent) -> Self {
    ident.name
  }
}

impl Display for LoxIdent {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.name)
  }
}
