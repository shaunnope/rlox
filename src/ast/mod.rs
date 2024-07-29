#[cfg(test)]
mod tests;

mod printer;

use std::fmt;
use crate::token::{Token, TokenType};

#[derive(Debug)]
pub enum Expr {
  Literal(TokenType),
  Grouping(Box<Expr>),
  Binary {
    left: Box<Expr>,
    op: Token,
    right: Box<Expr>
  },
  Unary {
    op: Token,
    right: Box<Expr>
  }
}

impl Expr {
  #[allow(dead_code)]
  /// Display for Reverse Polish Notation
  fn rpn(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Literal(token) => return write!(f, "{}", token),
      Self::Grouping(node) => return write!(f, "(group {})", node),
      Self::Binary{left, op, right} => {
        return write!(f, "{} {} {}", left, right, op)
      },
      Self::Unary{op, right} => {
        return write!(f, "{} {}", right, op)
      },
    }
  }
}

impl fmt::Display for Expr {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Literal(token) => return write!(f, "{}", token),
      Self::Grouping(node) => return write!(f, "(group {})", node),
      Self::Binary{left, op, right} => {
        return write!(f, "({} {} {})", op, left, right)
      },
      Self::Unary{op, right} => {
        return write!(f, "({} {})", op, right)
      },
    }
  }
}

