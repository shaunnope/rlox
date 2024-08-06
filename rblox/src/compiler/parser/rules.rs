
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
  Number
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
      T::Slash |
      T::Star => Self(F::None, F::Binary, P::Factor),
      T::Number(_) => Self(F::Number, F::None, P::None),
      _ => Self(F::None, F::None, P::None),
    }
  }
}