macro_rules! wrap {
  ($val:expr) => {{
    Ok(Some(Box::new($val)))
  }};
}

use crate::{
  ast::{
    expr::{self, Expr},
    stmt::{self, Stmt},
  },
  data::LoxValue,
  interpreter::{control_flow::ControlFlow, error::RuntimeError},
  span::Span,
  token::TokenType,
};

// use crate::error::{Error, PartialErr, Type};


#[allow(dead_code)]
pub fn evaluate(expression: &Expr) -> Result<Evaluation, Error> {
  match expression {
    Expr::Literal(token) => Ok(get_literal(token)),
    Expr::Grouping(expr) => evaluate(expr),
    Expr::Unary { op, right } => resolve_unary(op, right),
    Expr::Binary { left, op, right } => resolve_binary(left, op, right),
  }
}

fn get_literal(token: &TokenType) -> Evaluation {
  match token {
    TokenType::Nil => None,
    TokenType::Number(val) => Some(Box::new(*val)),
    TokenType::String(val) => Some(Box::new(val.to_string())),
    TokenType::True => Some(Box::new(true)),
    TokenType::False => Some(Box::new(false)),
    _ => Some(Box::new(token.lexeme())), // TODO: should be an error
  }
}

fn truth(val: &Evaluation) -> bool {
  match val {
    Some(val) => {
      if let Some(flag) = val.as_any().downcast_ref::<bool>() {
        return *flag;
      }
      true
    }
    None => false,
  }
}

fn equals(a: &Evaluation, b: &Evaluation) -> bool {
  match a {
    Some(a) => {
      if let Some(b) = b {
        if (&*a).type_id() != (&*b).type_id() {
          return false;
        }
        a == b
      } else {
        false
      }
    }
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
pub fn number(val: &Evaluation) -> Result<f64, PartialErr> {
  if let Some(res) = val {
    if let Some(num) = res.as_any().downcast_ref::<f64>() {
      return Ok(*num);
    }
  }
  Err(PartialErr::new(Type::Runtime, "Operand must be a number."))
}

fn numbers(left: &Evaluation, right: &Evaluation) -> Result<(f64, f64), PartialErr> {
  if let Ok(left) = number(left) {
    if let Ok(right) = number(right) {
      return Ok((left, right));
    }
    return Err(PartialErr::new(
      Type::Runtime,
      "Right operand is not a number.",
    ));
  }
  Err(PartialErr::new(
    Type::Runtime,
    "Left operand is not a number.",
  ))
}

fn resolve_unary(op: &Token, right: &Box<Expr>) -> Result<Evaluation, Error> {
  let right = &evaluate(right)?;

  match op.ttype {
    TokenType::Minus => match number(right) {
      Ok(num) => wrap!(-num),
      Err(err) => Err(op.error(err.err, &err.message)),
    },
    TokenType::Bang => {
      wrap!(!truth(right))
    }
    _ => Err(op.error(Type::Runtime, "Malformed unary expression.")),
  }
}

fn resolve_binary(left: &Box<Expr>, op: &Token, right: &Box<Expr>) -> Result<Evaluation, Error> {
  let left = &evaluate(left)?;
  let right = &evaluate(right)?;

  match op.ttype {
    TokenType::Minus => {
      match numbers(left, right) {
        Ok((left, right)) => wrap!(left - right),
        Err(err) => Err(op.error(err.err, &err.message)),
      }
      // Err(Box::new(
      //   LoxError::new(Type::Runtime, op.line, "",
      //   "Failed to perform subtraction.")))
    }
    TokenType::Slash => match numbers(left, right) {
      Ok((left, right)) => wrap!(left / right),
      Err(err) => Err(op.error(err.err, &err.message)),
    },
    TokenType::Star => match numbers(left, right) {
      Ok((left, right)) => wrap!(left * right),
      Err(err) => Err(op.error(err.err, &err.message)),
    },
    TokenType::Plus => process_addition(op, left, right),
    TokenType::Greater => match numbers(left, right) {
      Ok((left, right)) => wrap!(left > right),
      Err(err) => Err(op.error(err.err, &err.message)),
    },
    TokenType::GreaterEqual => match numbers(left, right) {
      Ok((left, right)) => wrap!(left >= right),
      Err(err) => Err(op.error(err.err, &err.message)),
    },
    TokenType::Less => match numbers(left, right) {
      Ok((left, right)) => wrap!(left < right),
      Err(err) => Err(op.error(err.err, &err.message)),
    },
    TokenType::LessEqual => match numbers(left, right) {
      Ok((left, right)) => wrap!(left < right),
      Err(err) => Err(op.error(err.err, &err.message)),
    },
    TokenType::BangEqual => wrap!(!equals(left, right)),
    TokenType::EqualEqual => wrap!(equals(left, right)),
    _ => Err(op.error(Type::Runtime, "Malformed binary expression.")),
  }
}

fn process_addition(
  op: &Token,
  left: &Evaluation,
  right: &Evaluation,
) -> Result<Evaluation, Error> {
  if let Ok((left, right)) = numbers(left, right) {
    return wrap!(left + right);
  }

  if let Some(res) = left {
    if let Some(left) = res.as_any().downcast_ref::<String>() {
      if let Some(res) = right {
        if let Some(right) = res.as_any().downcast_ref::<String>() {
          return wrap!(left.to_owned() + right);
        }
      }
    }
  }

  Err(op.error(
    Type::Runtime,
    "Operands must be two numbers or two strings.",
  ))
}
