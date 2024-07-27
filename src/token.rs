#![allow(dead_code)]

use std::fmt::{self, Display};

#[derive(Debug)]
pub enum TokenType {
  // single character
  LeftParen, RightParen, LeftBrace, RightBrace,
  Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

  // one, two chars
  Bang, BangEqual,
  Equal, EqualEqual,
  Greater, GreaterEqual,
  Less, LessEqual,

  // literals
  Identifier, String, Number,

  // keywords
  And, Class, Else, False, Fun, For, If, Nil, Or,
  Print, Return, Super, This, True, Var, While,

  Eof
}

impl Display for TokenType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

pub struct Token<T: Display> {
  pub ttype: TokenType,
  pub lexeme: String,
  pub literal: T,
  pub line: i32
}

impl<T: Display> Display for Token<T> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} {} {}", self.ttype, self.lexeme, self.literal)
  }
}