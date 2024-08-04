use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
  Number(f64)
}

impl Display for Value {
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
