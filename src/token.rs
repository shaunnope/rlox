#![allow(dead_code)]

use std::fmt::{self, Display};

#[cfg(test)]
mod tests;


#[derive(Debug, PartialEq)]
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

// impl TokenType {
//   pub fn get_literal(&self) -> &str {
//     match self {
//       Self::LeftParen => "(",
//       Self::RightParen => ")",
//       Self::LeftBrace => "{",
//       Self::RightBrace => "}",
//       Self::Comma => ",",
//       Self::Dot => ".",
//       Self::Minus => "-",
//       Self::Plus => "+",
//       Self::Semicolon => ";",
//       Self::Slash => "\\",
//       Self::Star => "*",

//       Self::Bang => "!",
//       Self::BangEqual => "!=",
//       Self::Equal => "=",
//       Self::EqualEqual => "==",
//       Self::Greater => ">",
//       Self::GreaterEqual => ">=",
//       Self::Less => "<",
//       Self::LessEqual => "<=",

//       Self::Identifier(s) => s.as_str(),
//       Self::String(s) => s.as_str(),
//       Self::Integer(n) => format!("{n}"),
//       Self::Float(n) => n.to_string().as_str(),
      
//       Self::And => "and",
//       Self::Class => "class",
//       Self::Else => "else",
//       Self::False => "false",
//       Self::Fun => "fun",
//       Self::For => "for",
//       Self::If => "if",
//       Self::Nil => "nil",
//       Self::Or => "or",
//       Self::Print => "print",
//       Self::Return => "return",
//       Self::Super => "super",
//       Self::This => "this",
//       Self::True => "true",
//       Self::Var => "let",
//       Self::While => "while",
//       Self::EOF => ""
//     }
//   }
// }

impl Display for TokenType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:?}", self)
  }
}

#[derive(Debug, PartialEq)]
pub struct Token {
  pub ttype: TokenType,
  pub line: i32
}

impl Token {
  pub fn new(line: i32) -> Self {
    Token {ttype: TokenType::Nil, line}
  }
}

impl Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} {}", self.ttype, self.line)
  }
}