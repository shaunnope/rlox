use std::{collections::HashMap, mem, rc::Rc};

use crate::{
  ast::{
    expr::{self, Expr},
    stmt::{self, Stmt},
  },
  data::{LoxClass, LoxFunction, LoxIdent, LoxIdentId, LoxValue, LoxInstance},
  interpreter::{control_flow::ControlFlow, environment::Environment, error::RuntimeError},
  span::Span,
  token::TokenType,
};

pub mod control_flow;
pub mod environment;
pub mod error;

mod native;

#[derive(Debug)]
pub struct Interpreter {
  locals: HashMap<LoxIdentId, usize>,
  pub globals: Environment,
  env: Environment,
}

impl Interpreter {
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
      FunDecl(fun) => self.eval_fun_decl(fun),
      ClassDecl(class) => self.eval_class_decl(class),
      If(if_stmt) => self.eval_if_stmt(if_stmt),
      While(while_stmt) => self.eval_while_stmt(while_stmt),
      Print(print) => self.eval_print_stmt(print),
      Return(ret) => self.eval_return_stmt(ret),
      Block(block) => self.eval_block(&block.stmts, Environment::new_enclosed(&self.env)),
      Expr(expr) => self.eval_expr(&expr.expr).map(drop),
      Dummy(_) => unreachable!(),
      // _ => Ok(()),
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

  fn eval_fun_decl(&mut self, fun: &stmt::FunDecl) -> CFResult<()> {
    self.env.define(
      fun.name.clone(),
      LoxValue::Function(Rc::new(LoxFunction {
        decl: Rc::new(fun.clone()),
        closure: self.env.clone(),
        is_class_init: false,
      })),
    );
    Ok(())
  }

  fn eval_class_decl(&mut self, decl: &stmt::ClassDecl) -> CFResult<()> {
    let super_class = decl.super_name.as_ref()
      .map(|name| {
        let maybe_class = self.lookup_variable(name)?;
        if let LoxValue::Class(class) = maybe_class {
          Ok(class)
        } else {
          Err(ControlFlow::from(RuntimeError::UnsupportedType { 
            message: format!("Superclass must be a class: got {}", maybe_class), 
            span: name.span 
          }))
        }
      })
      .transpose()?;

    if let Some(super_class) = super_class.clone() {
      self.env = Environment::new_enclosed(&self.env);
      self.env.define("super", LoxValue::Class(super_class));
    }
    
    let methods = decl.methods.iter().cloned()
      .map(|decl| {
        (
          decl.name.name.clone(),
          Rc::new(LoxFunction {
            is_class_init: decl.name.name == "init",
            decl: Rc::new(decl),
            closure: self.env.clone()
          })
        )
      }).collect();

    if super_class.is_some() {
      self.env = self.env.enclosed().unwrap();
    }

    self.env.define(
      decl.name.clone(),
      LoxValue::Class(Rc::new(LoxClass {
          name: decl.name.clone(),
          super_class,
          methods,
      })),
    );

    Ok(())
  }


  fn eval_if_stmt(&mut self, stmt: &stmt::If) -> CFResult<()> {
    if self.eval_expr(&stmt.cond)?.truth() {
      self.eval_stmt(&stmt.then_branch)?;
    } else if let Some(br) = &stmt.else_branch {
      self.eval_stmt(br)?;
    }
    Ok(())
  }

  fn eval_while_stmt(&mut self, stmt: &stmt::While) -> CFResult<()> {
    while self.eval_expr(&stmt.cond)?.truth() {
      self.eval_stmt(&stmt.body)?;
    }
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

  fn eval_return_stmt(&mut self, stmt: &stmt::Return) -> CFResult<()> {
    let value = match &stmt.value {
      Some(expr) => self.eval_expr(expr)?,
      None => LoxValue::Nil,
    };

    Err(ControlFlow::Return(value))
  }

  pub(crate) fn eval_block(&mut self, block: &[Stmt], new_env: Environment) -> CFResult<()> {
    let old_env = mem::replace(&mut self.env, new_env);
    let result = self.eval_stmts(&block);
    self.env = old_env;
    result
  }

  fn eval_expr(&mut self, expr: &Expr) -> CFResult<LoxValue> {
    use Expr::*;
    match &expr {
      Var(var) => self.eval_var_expr(var),
      Call(call) => self.eval_call_expr(call),
      Get(get) => self.eval_get_expr(get),
      Set(set) => self.eval_set_expr(set),
      This(this) => self.lookup_variable(&this.name),
      Super(sup) => self.eval_super_expr(sup),
      Lit(lit) => self.eval_lit_expr(lit),
      Group(group) => self.eval_group_expr(group),
      Unary(unary) => self.eval_unary_expr(unary),
      Binary(binary) => self.eval_binary_expr(binary),
      Logical(logical) => self.eval_logical_expr(logical),
      Assignment(assign) => self.eval_assignment(assign),
      Lambda(lambda) => self.eval_lambda(lambda),
    }
  }

  fn eval_var_expr(&mut self, var: &expr::Var) -> CFResult<LoxValue> {
    Ok(self.lookup_variable(&var.name)?)
  }

  fn eval_call_expr(&mut self, call: &expr::Call) -> CFResult<LoxValue> {
    use LoxValue::*;
    let callee = self.eval_expr(&call.callee)?;

    let args = call
      .args
      .iter()
      .map(|expr| self.eval_expr(expr))
      .collect::<Result<Vec<_>, _>>()?;

    let callable = match callee {
      Function(callable) => callable,
      Class(class) => class,
      _ => {
        return Err(ControlFlow::from(RuntimeError::UnsupportedType {
          message: format!(
            "Type `{}` is not callable. Can only call functions",
            callee.type_name()
          ),
          span: call.span,
        }))
      }
    };

    if callable.arity() != args.len() {
      return Err(ControlFlow::from(RuntimeError::UnsupportedType {
        message: format!(
          "Expected {} arguments, but got {}",
          callable.arity(),
          args.len()
        ),
        span: call.span,
      }));
    }

    callable.call(self, &args)
  }

  fn eval_get_expr(&mut self, get: &expr::Get) -> CFResult<LoxValue> {
    let maybe_obj = self.eval_expr(&get.obj)?;
    let obj  = Self::ensure_object(maybe_obj, get.name.span)?;
    Ok(obj.get(&get.name)?)
  }

  fn eval_set_expr(&mut self, set: &expr::Set) -> CFResult<LoxValue> {
    let maybe_obj = self.eval_expr(&set.obj)?;
    let obj  = Self::ensure_object(maybe_obj, set.name.span)?;
    let value = self.eval_expr(&set.value)?;
    obj.set(&set.name, value.clone());
    Ok(value)
  }

  fn eval_super_expr(&mut self, sup: &expr::Super) -> CFResult<LoxValue> {
    // FOllowing two unwraps should never fail due to semantic verification
    let dist = self.locals.get(&sup.super_ident.id).unwrap();
    let super_class = self.env
      .read_at(*dist, "super")
      .as_class()
      .unwrap();

    // The environment where "this" is defined is always bound immediately inside the
    // environment that defined "super" (the "this env" encloses the "super env").
    let this = self.env
      .read_at(dist - 1, "this")
      .as_object()
      .unwrap();

      match super_class.get_method(&sup.method) {
        Some(method) => Ok(
          LoxValue::Function(
          method.bind(&this))
        ),
        None => Err(ControlFlow::from(
          RuntimeError::UndefinedProperty {
          ident: sup.method.clone(),
        })),
      }
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
    match &logical.operator.kind {
      TokenType::And if !left.truth() => Ok(left),
      TokenType::Or if left.truth() => Ok(left),
      _ => self.eval_expr(&logical.right),
    }
  }

  fn eval_assignment(&mut self, assign: &expr::Assignment) -> CFResult<LoxValue> {
    let value = self.eval_expr(&assign.value)?;

    if let Some(dist) = self.locals.get(&assign.name.id) {
      Ok(self.env.assign_at(*dist, &assign.name, value))
    } else {
      Ok(self.globals.assign(&assign.name, value)?)
    }
  }

  fn eval_lambda(&mut self, lambda: &expr::Lambda) -> CFResult<LoxValue> {
    self.eval_fun_decl(&lambda.decl)?;

    // return identifier to function
    Ok(self.env.read(&lambda.decl.name)?)
  }
}

impl Interpreter {
  pub fn new() -> Self {
    let mut globals = Environment::new();
    native::attach(&mut globals);

    Self {
      env: globals.clone(),
      globals,
      locals: HashMap::new(),
    }
  }

  pub fn resolve_local(&mut self, ident: &LoxIdent, depth: usize) {
    self.locals.insert(ident.id, depth);
  }

  fn lookup_variable(&self, ident: &LoxIdent) -> CFResult<LoxValue> {
    if let Some(distance) = self.locals.get(&ident.id) {
      Ok(self.env.read_at(*distance, ident))
    } else {
      Ok(self.globals.read(ident)?)
    }
  }

  fn ensure_object(value: LoxValue, error_span: Span) -> CFResult<Rc<LoxInstance>> {
    if let LoxValue::Object(instance) = value {
      Ok(instance)
    } else {
      Err(RuntimeError::UnsupportedType {
        message: "Only objects can have properties".into(),
        span: error_span,
      }
      .into())
    }
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
