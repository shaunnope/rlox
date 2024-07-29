#[cfg(test)]
mod tests;


use std::{any::Any, fmt};
use crate::token::{Token, TokenType};
use crate::error::{Error, Type, PartialErr};

use crate::DynEq;


macro_rules! wrap {
  ($val:expr) => {
    {
      Ok(Some(Box::new($val)))
    }
  };
}

/// A dynamic type for evaluated results.
/// 
/// None for lox's nil
type Evaluation = Option<Box<dyn DynEq>>;

#[derive(Debug)]
pub enum Expr {
  Literal(TokenType),
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

#[allow(dead_code)]
impl Expr {
  
  /// Display for Reverse Polish Notation
  fn rpn(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Literal(token) => write!(f, "{}", token),
      Self::Grouping(node) => write!(f, "(group {})", node),
      Self::Binary{left, op, right} => {
        write!(f, "{} {} {}", left, right, op)
      },
      Self::Unary{op, right} => {
        write!(f, "{} {}", right, op)
      },
    }
  }

  fn evaluate(&self) -> Result<Evaluation, Error> {
    match self {
      Self::Literal(token) => Ok(get_literal(token)),
      Self::Grouping(expr) => expr.evaluate(),
      Self::Unary { op, right } => resolve_unary(op, right),
      Self::Binary { left, op, right } => resolve_binary(left, op, right),
      // _ => Err(Box::new(LoxError::new(Type::Runtime, -1, "", "Unable to evaluate expression.")))
    }
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

fn get_literal(token: &TokenType) -> Evaluation {
  match token {
    TokenType::Nil
    => None,
    TokenType::Number(val)
    => Some(Box::new(*val)),
    TokenType::String(val)
    => Some(Box::new(val.to_string())),
    TokenType::True
    => Some(Box::new(true)),
    TokenType::False
    => Some(Box::new(false)),
    _ => Some(Box::new(token.lexeme())), // TODO: should be an error
  }
}

fn truth(val: &Evaluation) -> bool {
  match val {
    Some(val) => {
      if let Some(flag) = val.as_any().downcast_ref::<bool>() {
        return *flag
      }
      true
    },
    None => false
  }
}

fn equals(a: &Evaluation, b: &Evaluation) -> bool {
  match a {
    Some(a) => {
      if let Some(b) = b {
        if (&*a).type_id() != (&*b).type_id() {
          return false
        }
        a == b
      } else {
        false
      }
    },
    None => {
      if let None = b {
        true
      } else {
        false
      }
    }
  }
}

/// Downcast to number
fn number(val: &Evaluation) -> Result<f64, PartialErr> {
  if let Some(res) = val {
    if let Some(num) = res.as_any().downcast_ref::<f64>() {
      return Ok(*num)
    }
  }
  Err(PartialErr::new(Type::Runtime, "Operand must be a number."))
}

fn numbers(left: &Evaluation, right: &Evaluation) -> Result<(f64, f64), PartialErr> {
  if let Ok(left) = number(left) {
    if let Ok(right) = number(right) {
      return Ok((left, right))
    }
    return Err(PartialErr::new(Type::Runtime, "Right operand is not a number."))
  }
  Err(PartialErr::new(Type::Runtime, "Left operand is not a number."))
}

fn resolve_unary(op: &Token, right: &Box<Expr>) -> Result<Evaluation, Error> {
  let right = &right.evaluate()?;
  
  match op.ttype {
    TokenType::Minus => {
      match number(right) {
        Ok(num) => wrap!(-num),
        Err(err) => Err(op.error(err.err, &err.message))
      }
    },
    TokenType::Bang => {
      wrap!(!truth(right))
    },
    _ => Err(op.error(Type::Runtime, "Malformed unary expression."))
  }
}

fn resolve_binary(left: &Box<Expr>, op: &Token, right: &Box<Expr>) -> Result<Evaluation, Error> {
  let left = &left.evaluate()?;
  let right = &right.evaluate()?;
  
  match op.ttype {
    TokenType::Minus => {
      match numbers(left, right) {
        Ok((left, right)) => wrap!(left - right),
        Err(err) => Err(op.error(err.err, &err.message))
      }
      // Err(Box::new(
      //   LoxError::new(Type::Runtime, op.line, "", 
      //   "Failed to perform subtraction.")))
    },
    TokenType::Slash => {
      match numbers(left, right) {
        Ok((left, right)) => wrap!(left / right),
        Err(err) => Err(op.error(err.err, &err.message))
      }
    },
    TokenType::Star => {
      match numbers(left, right) {
        Ok((left, right)) => wrap!(left * right),
        Err(err) => Err(op.error(err.err, &err.message))
      }
    },
    TokenType::Plus => process_addition(op, left, right),
    TokenType::Greater => {
      match numbers(left, right) {
        Ok((left, right)) => wrap!(left > right),
        Err(err) => Err(op.error(err.err, &err.message))
      }
    },
    TokenType::GreaterEqual => {
      match numbers(left, right) {
        Ok((left, right)) => wrap!(left >= right),
        Err(err) => Err(op.error(err.err, &err.message))
      }
    },
    TokenType::Less => {
      match numbers(left, right) {
        Ok((left, right)) => wrap!(left < right),
        Err(err) => Err(op.error(err.err, &err.message))
      }
    },
    TokenType::LessEqual => {
      match numbers(left, right) {
        Ok((left, right)) => wrap!(left < right),
        Err(err) => Err(op.error(err.err, &err.message))
      }
    },
    TokenType::BangEqual => wrap!(!equals(left, right)),
    TokenType::EqualEqual => wrap!(equals(left, right)),
    _ => Err(op.error(Type::Runtime, "Malformed binary expression."))
  }
}

fn process_addition(op: &Token, left: &Evaluation, right: &Evaluation) -> Result<Evaluation, Error> {
  if let Ok((left, right)) = numbers(left, right) {
    return wrap!(left + right)
  }

  if let Some(res) = left {
    if let Some(left) = res.as_any().downcast_ref::<String>() {
      if let Some(res) = right {
        if let Some(right) = res.as_any().downcast_ref::<String>() {
          return wrap!(left.to_owned() + right)
        }
      }
    }
  }

  Err(op.error(Type::Runtime, "Operands must be two numbers or two strings."))
}