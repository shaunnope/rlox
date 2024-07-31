#![allow(dead_code)]

use std::fmt::{self, Display};

use crate::parser::scanner::error::ScanError;
use crate::span::Span;

// #[cfg(test)]
// mod tests;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenType {
  // single character
  LeftParen,
  RightParen,
  LeftBrace,
  RightBrace,
  Comma,
  Dot,
  Minus,
  Plus,
  Semicolon,
  Star,

  // one, two chars
  Slash,
  Comment(String),
  BlockComment(String),
  Bang,
  BangEqual,
  Equal,
  EqualEqual,
  Greater,
  GreaterEqual,
  Less,
  LessEqual,

  // literals
  Identifier(String),
  String(String),
  Number(f64),
  Whitespace(String),

  // keywords
  And,
  Class,
  Else,
  False,
  Fun,
  For,
  If,
  Nil,
  Or,
  Print,
  Return,
  Super,
  This,
  True,
  Var,
  While,

  EOF,

  Dummy,
  Error(ScanError),
}

impl TokenType {
  pub fn new() -> Self {
    Self::Nil
  }

  pub fn lexeme(&self) -> String {
    format!("{}", self)
  }

  pub fn get_pair(&self) -> TokenType {
    use TokenType::*;
    match self {
      LeftParen => RightParen,
      RightParen => LeftParen,
      LeftBrace => RightBrace,
      RightBrace => LeftBrace,
      unexpected => panic!(
        "Token `{:?}` does not have a pair. This is a bug.",
        unexpected
      ),
    }
  }
}

impl From<&str> for TokenType {
  fn from(value: &str) -> Self {
    use TokenType::*;
    match value {
      "nil" => Nil,
      "true" => True,
      "false" => False,
      "this" => This,
      "super" => Super,
      "class" => Class,
      "and" => And,
      "or" => Or,
      "if" => If,
      "else" => Else,
      "return" => Return,
      "fun" => Fun,
      "for" => For,
      "while" => While,
      "var" => Var,
      "print" => Print,
      // "typeof" => Typeof,
      // "show" => Show,
      identifier => Identifier(identifier.to_string()),
    }
  }
}

impl Display for TokenType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    use TokenType::*;
    match self {
      // literals
      Identifier(s) => s.fmt(f),
      String(s) => write!(f, "\"{}\"", s),
      Number(n) => n.fmt(f),

      // symbols
      LeftParen => f.write_str("("),
      RightParen => f.write_str(")"),
      LeftBrace => f.write_str("{"),
      RightBrace => f.write_str("}"),
      Comma => f.write_str(","),
      Dot => f.write_str("."),
      Minus => f.write_str("-"),
      Plus => f.write_str("+"),
      Semicolon => f.write_str(";"),
      Slash => f.write_str("/"),
      Star => f.write_str("*"),
      Bang => f.write_str("!"),
      BangEqual => f.write_str("!="),
      Equal => f.write_str("="),
      EqualEqual => f.write_str("=="),
      Greater => f.write_str(">"),
      GreaterEqual => f.write_str(">="),
      Less => f.write_str("<"),
      LessEqual => f.write_str("<="),

      // keywords
      And => f.write_str("and"),
      Class => f.write_str("class"),
      Else => f.write_str("else"),
      False => f.write_str("false"),
      Fun => f.write_str("fun"),
      For => f.write_str("for"),
      If => f.write_str("if"),
      Nil => f.write_str("nil"),
      Or => f.write_str("or"),
      Print => f.write_str("print"),
      Return => f.write_str("return"),
      Super => f.write_str("super"),
      This => f.write_str("this"),
      True => f.write_str("true"),
      Var => f.write_str("var"),
      While => f.write_str("while"),
      EOF => f.write_str("<eof>"),

      Dummy => f.write_str("<dummy>"),
      _ => f.write_str("<unimpl>"),
    }
  }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
  pub kind: TokenType,
  pub span: Span,
}

impl Token {
  pub fn new(kind: TokenType, span: Span) -> Self {
    Self { kind, span }
  }

  pub fn dummy() -> Self {
    Self::new(TokenType::Dummy, Span::new(0, 0))
  }
}

impl Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    self.kind.fmt(f)
  }
}
