use std::{
  fmt::Debug,
  ops::Neg
};

#[derive(Clone, PartialEq)]
pub enum Value {
  Number(f64)
}

impl Debug for Value {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Value::*;
    match self {
      Number(n) => {
        if n.floor() == *n {
          write!(f, "{:.0}", n)
        } else {
          write!(f, "{}", n)
        }
      },
    }
  }
}

impl Neg for Value {
  type Output = Self;
  fn neg(self) -> Self::Output {
    use Value::*;
    match self {
      Number(n) => Number(-n)
    }
  }
}

impl From<f64> for Value {
  fn from(value: f64) -> Self {
    Self::Number(value)
  }
}