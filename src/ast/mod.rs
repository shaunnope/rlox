mod printer;

use std::fmt;
use crate::token::{Token, TokenType};

#[derive(Debug)]
pub enum Expr {
  Literal(Token),
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


pub fn example() {
  let expression = Expr::Binary {
      left: Box::new(Expr::Unary {
          op: Token {
              ttype: TokenType::Minus,
              line: 1
          },
          right: Box::new(Expr::Literal(
              Token {
                  ttype: TokenType::Number(123.0),
                  line: 1
              }
          ))
      }),
      op: Token {
          ttype: TokenType::Star,
          line: 1
      },
      right: Box::new(Expr::Grouping(
          Box::new(Expr::Literal(
              Token { 
                  ttype: TokenType::Number(45.67), 
                  line: 1 
              }
          ))
      )),
  };

  println!("{expression}")
}
