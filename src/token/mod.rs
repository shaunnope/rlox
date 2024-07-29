#![allow(dead_code)]

use std::fmt::{self, Display};

use crate::error::ParseError;

#[cfg(test)]
mod tests;


#[derive(Debug, PartialEq, Clone)]
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
  Identifier(String), String(String), Number(f64),

  // keywords
  And, Class, Else, False, Fun, For, If, Nil, Or,
  Print, Return, Super, This, True, Var, While,

  EOF
}

impl TokenType {
  pub fn new() -> Self {
    Self::Nil
  }

  pub fn lexeme(&self) -> String {
    match self {
      Self::LeftParen => "(".to_string(),
      Self::RightParen => ")".to_string(),
      Self::LeftBrace => "{".to_string(),
      Self::RightBrace => "}".to_string(),
      Self::Comma => ",".to_string(),
      Self::Dot => ".".to_string(),
      Self::Minus => "-".to_string(),
      Self::Plus => "+".to_string(),
      Self::Semicolon => ";".to_string(),
      Self::Slash => "/".to_string(),
      Self::Star => "*".to_string(),
      Self::Bang => "!".to_string(),
      Self::BangEqual => "!=".to_string(),
      Self::Equal => "=".to_string(),
      Self::EqualEqual => "==".to_string(),
      Self::Greater => ">".to_string(),
      Self::GreaterEqual => ">=".to_string(),
      Self::Less => "<".to_string(),
      Self::LessEqual => "<=".to_string(),

      Self::Identifier(s) => s.to_string(),
      Self::String(s) => s.to_string(),
      Self::Number(n) => n.to_string(),

      Self::And => "and".to_string(),
      Self::Class => "class".to_string(),
      Self::Else => "else".to_string(),
      Self::False => "false".to_string(),
      Self::Fun => "fun".to_string(),
      Self::For => "for".to_string(),
      Self::If => "if".to_string(),
      Self::Nil => "nil".to_string(),
      Self::Or => "or".to_string(),
      Self::Print => "print".to_string(),
      Self::Return => "return".to_string(),
      Self::Super => "super".to_string(),
      Self::This => "this".to_string(),
      Self::True => "true".to_string(),
      Self::Var => "let".to_string(),
      Self::While => "while".to_string(),
      Self::EOF => "end".to_string()
    }
  }
}

impl Display for TokenType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.lexeme())
  }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
  pub ttype: TokenType,
  pub line: i32
}

impl Token {
  pub fn new(line: i32) -> Self {
    Token {ttype: TokenType::Nil, line}
  }

  pub fn error(&self, message: &str) -> Box<ParseError> {
    let error = ParseError::new(self.line, &format!(" at {}", 
    if self.ttype == TokenType::EOF {
      "end".to_string()
    } else { format!("'{}'", self.ttype.lexeme())}), message);
    error.display();
    Box::new(error)
  }
}

impl Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.ttype)
  }
}