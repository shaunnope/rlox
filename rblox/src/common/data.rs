use std::{fmt::Display, hash::Hash, mem};

use crate::compiler::{
  parser::error::ParseError,
  scanner::token::{Token, TokenType}
};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LoxObject {
  Identifier(String),
  String(String),
}

impl LoxObject {
  /// Returns the canonical type name.
  pub fn type_name(&self) -> &'static str {
    use LoxObject::*;
    match self {
      Identifier(_) => "<ident>",
      String(_) => "string",
      // Function(_) => "<func>",
      // Class(_) => "<class>",
      // Object(_) => "<instance>",
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
      Identifier(s) => write!(f, "{s}"),
      String(s) => write!(f, "{s}"),
    }
  }
}

impl TryFrom<Token> for LoxObject {
  type Error = ParseError;
  fn try_from(value: Token) -> Result<Self, Self::Error> {
    match value.kind {
      TokenType::Identifier(s) => Ok(LoxObject::Identifier(s)),
      _ => Err(ParseError::UnexpectedToken { 
        message: "Expected identifier".into(), 
        offending: value, 
        expected: Some(TokenType::Identifier("<ident>".into()))
      }) 
    }
  }
}