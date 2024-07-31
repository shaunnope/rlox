use std::{collections::HashMap, mem, rc::Rc};

use crate::{
  ast::{
    expr::{self, Expr},
    stmt::{self, Stmt},
  },
  data::LoxValue,
  interpreter::{control_flow::ControlFlow, environment::Environment, error::RuntimeError},
  span::Span,
  token::TokenType,
};

pub mod control_flow;
pub mod environment;
pub mod error;

// mod expr;

#[derive(Debug)]
pub struct Interpreter {
  // locals: HashMap<LoxIdentId, usize>,
  // globals: Environment,
  env: Environment,
}

impl Interpreter {
  pub fn new() -> Self {
    Self {
      env: Environment::new(),
    }
  }

  // Note that `CFResult` must not be exposed to the interpreter caller.
  // It is an implementation detail.
  pub fn interpret(&mut self, stmts: &[Stmt]) -> Result<(), RuntimeError> {
    match self.eval_stmts(stmts) {
      Ok(()) => Ok(()),
      Err(ControlFlow::Err(err)) => Err(err),
      Err(ControlFlow::Return(_)) => unreachable!(),
    }
  }

  //
  // Statements
  //

  fn eval_stmts(&mut self, stmts: &[Stmt]) -> CFResult<()> {
    for stmt in stmts {
      self.eval_stmt(stmt)?;
    }
    Ok(())
  }

  fn eval_stmt(&mut self, stmt: &Stmt) -> CFResult<()> {
    use Stmt::*;
    match &stmt {
      VarDecl(var) => self.eval_var_decl(var),
      Print(print) => self.eval_print_stmt(print),
      Block(block) => self.eval_block(block, Environment::new_enclosed(&self.env)),
      Expr(expr) => self.eval_expr(&expr.expr).map(drop),
      Dummy(_) => unreachable!(),
      _ => Ok(()),
    }
  }

  fn eval_var_decl(&mut self, var: &stmt::VarDecl) -> CFResult<()> {
    let mut value = LoxValue::Unset;
    if let Some(init) = &var.init {
      value = self.eval_expr(init)?;
    }

    self.env.define(var.name.clone(), value);

    Ok(())
  }

  fn eval_print_stmt(&mut self, print: &stmt::Print) -> CFResult<()> {
    let val = self.eval_expr(&print.expr)?;
    match print.debug {
      true => println!("{:?}", val),
      false => println!("{}", val),
    }
    Ok(())
  }

  fn eval_block(&mut self, block: &stmt::Block, new_env: Environment) -> CFResult<()> {
    let old_env = mem::replace(&mut self.env, new_env);
    let result = self.eval_stmts(&block.stmts);
    self.env = old_env;
    result
  }

  fn eval_expr(&mut self, expr: &Expr) -> CFResult<LoxValue> {
    use Expr::*;
    match &expr {
      Var(var) => self.eval_var_expr(var),
      Lit(lit) => self.eval_lit_expr(lit),
      Group(group) => self.eval_group_expr(group),
      Unary(unary) => self.eval_unary_expr(unary),
      Binary(binary) => self.eval_binary_expr(binary),
      Logical(logical) => self.eval_logical_expr(logical),
      Assignment(assign) => self.eval_assignemnt(assign),
    }
  }

  fn eval_var_expr(&mut self, var: &expr::Var) -> CFResult<LoxValue> {
    Ok(self.env.read(&var.name)?)
  }

  fn eval_lit_expr(&mut self, lit: &expr::Lit) -> CFResult<LoxValue> {
    Ok(lit.value.clone())
  }

  fn eval_group_expr(&mut self, group: &expr::Group) -> CFResult<LoxValue> {
    self.eval_expr(&group.expr)
  }

  fn eval_unary_expr(&mut self, unary: &expr::Unary) -> CFResult<LoxValue> {
    let operand = self.eval_expr(&unary.operand)?;
    match &unary.operator.kind {
      TokenType::Minus => match operand {
        LoxValue::Number(n) => Ok(LoxValue::Number(-n)),
        unexpected => Err(
          RuntimeError::UnsupportedType {
            message: format!(
              "Bad type for unary `-` operator: `{}`",
              unexpected.type_name()
            ),
            span: unary.operator.span,
          }
          .into(),
        ),
      },
      TokenType::Bang => Ok(LoxValue::Boolean(!operand.truth())),
      unexpected => unreachable!("Invalid unary operator ({:?}).", unexpected),
    }
  }

  fn eval_binary_expr(&mut self, binary: &expr::Binary) -> CFResult<LoxValue> {
    use LoxValue::*;
    let left = self.eval_expr(&binary.left)?;
    let right = self.eval_expr(&binary.right)?;

    match &binary.operator.kind {
      TokenType::EqualEqual => Ok(LoxValue::Boolean(left.equals(&right))),
      TokenType::BangEqual => Ok(LoxValue::Boolean(!left.equals(&right))),

      TokenType::Greater => bin_cmp_op!(left > right, binary.operator),
      TokenType::GreaterEqual => bin_cmp_op!(left >= right, binary.operator),
      TokenType::Less => bin_cmp_op!(left < right, binary.operator),
      TokenType::LessEqual => bin_cmp_op!(left <= right, binary.operator),

      TokenType::Minus => bin_num_op!(left - right, binary.operator),
      TokenType::Star => bin_num_op!(left * right, binary.operator),
      TokenType::Slash => {
        // TODO: enable/disable division by zero with env var
        if let Number(divisor) = right {
          if divisor == 0.0 {
            return Err(
              RuntimeError::ZeroDivision {
                span: binary.operator.span,
              }
              .into(),
            );
          }
        }
        bin_num_op!(left / right, binary.operator)
      }

      TokenType::Plus => match (left, right) {
        (Number(left), Number(right)) => Ok(Number(left + right)),
        (String(left), String(right)) => Ok(String(left + &right)),
        // extended string concat
        (String(left), right) => Ok(String(left + &right.to_string())),
        (left, right) => Err(
          RuntimeError::UnsupportedType {
            message: format!(
              "Binary `+` operator can only operate over two numbers or two strings. \
            Got types `{}` and `{}`",
              left.type_name(),
              right.type_name()
            ),
            span: binary.operator.span,
          }
          .into(),
        ),
      },
      TokenType::Comma => Ok(right),

      unexpected => unreachable!("Invalid binary operator ({:?}).", unexpected),
    }
  }

  fn eval_logical_expr(&mut self, logical: &expr::Logical) -> CFResult<LoxValue> {
    let left = self.eval_expr(&logical.left)?;
    Ok(left)
  }

  fn eval_assignemnt(&mut self, assign: &expr::Assignment) -> CFResult<LoxValue> {
    let value = self.eval_expr(&assign.value)?;
    self.env.assign(&assign.name, value.clone())?;

    Ok(value)
  }
}

/// Control flow result
pub type CFResult<T> = Result<T, ControlFlow<LoxValue, RuntimeError>>;

macro_rules! bin_num_op {
  ( $left:tt $op:tt $right:tt, $op_token:expr ) => {
    match ($left, $right) {
      (Number(left), Number(right)) => Ok(Number(left $op right)),
      (left, right) => Err(RuntimeError::UnsupportedType {
        message: format!(
          "Binary `{}` operator can only operate over two numbers. \
          Got types `{}` and `{}`",
          stringify!($op),
          left.type_name(),
          right.type_name()
        ),
        span: $op_token.span
      }
      .into()),
    }
  };
}
use bin_num_op;

macro_rules! bin_cmp_op {
  ( $left:tt $op:tt $right:tt, $op_token:expr ) => {
    match ($left, $right) {
      (Number(left), Number(right)) => Ok(LoxValue::Boolean(left $op right)),
      (String(left), String(right)) => Ok(LoxValue::Boolean(left $op right)),
      (left, right) => Err(RuntimeError::UnsupportedType {
        message: format!(
          "Binary `{}` operator can only compare two numbers or two strings. \
          Got types `{}` and `{}`",
          stringify!($op),
          left.type_name(),
          right.type_name()
        ),
        span: $op_token.span,
      }
      .into()),
    }
  };
}
use bin_cmp_op;
