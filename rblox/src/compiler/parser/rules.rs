
use crate::compiler::scanner::token::TokenType;

  #[derive(Debug, Clone, PartialEq, PartialOrd)]
  pub enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparision,
    Term,
    Factor,
    Unary,
    Call,
    Primary
  }
  
  impl From<usize> for Precedence {
    fn from(value: usize) -> Self {
      use Precedence::*;
      match value {
        1 => Assignment,
        2 => Or,
        3 => And,
        4 => Equality,
        5 => Comparision,
        6 => Term,
        7 => Factor,
        8 => Unary,
        9 => Call,
        10 => Primary,
        _ => None
      }
    }
  }
  
  impl Precedence {
    pub fn update(&self, val: isize) -> Self {
      Self::from(((self.clone() as isize) + val) as usize)
    }
  }

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ParseFn {
  None,
  Group,
  Binary,
  Unary,
  Number,
  Literal,
  String,
  Variable,
  And, Or
}

pub struct ParseRule(pub ParseFn, pub ParseFn, pub Precedence);

impl From<&TokenType> for ParseRule {
  fn from(value: &TokenType) -> Self {
    use TokenType as T;
    use ParseFn  as F;
    use Precedence as P;
    match value {
      T::EOF => Self(F::None, F::None, P::None),
      T::LeftParen => Self(F::Group, F::None, P::None),

      T::Minus => Self(F::Unary, F::Binary, P::Term),
      T::Plus => Self(F::None, F::Binary, P::Term),
      T::Slash | T::Star
      => Self(F::None, F::Binary, P::Factor),

      T::Bang => Self(F::Unary, F::None, P::None),
      T::BangEqual | T::EqualEqual 
      => Self(F::None, F::Binary, P::Equality),

      T::Greater | T::GreaterEqual |
      T::Less | T::LessEqual 
      => Self(F::None, F::Binary, P::Comparision),

      T::And => Self(F::None, F::And, Precedence::And),
      T::Or => Self(F::None, F::Or, Precedence::Or),

      T::Number(_) => Self(F::Number, F::None, P::None),
      T::True | T::False | T::Nil => Self(F::Literal, F::None, P::None),
      T::String(_) => Self(F::String, F::None, P::None),
      T::Identifier(_) => Self(F::Variable, F::None, P::None),

      _ => Self(F::None, F::None, P::None),
    }
  }
}