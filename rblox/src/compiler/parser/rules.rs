
use crate::compiler::scanner::token::TokenType;

  #[derive(Debug, Clone, PartialEq, PartialOrd)]
  pub enum Precedence {
    None,
    Sequence,
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
        1 => Sequence,
        2 => Assignment,
        3 => Or,
        4 => And,
        5 => Equality,
        6 => Comparision,
        7 => Term,
        8 => Factor,
        9 => Unary,
        10 => Call,
        11 => Primary,
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
  Call,
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
      T::LeftParen => Self(F::Group, F::Call, P::Call),

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

      T::Comma => Self(F::None, F::Binary, P::Sequence),

      _ => Self(F::None, F::None, P::None),
    }
  }
}