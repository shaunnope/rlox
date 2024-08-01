use std::{
  fmt::{self, Debug, Display},
  rc::Rc,
  sync::atomic::{self, AtomicUsize},
};

use crate::{
  ast::stmt::FunDecl,
  interpreter::{control_flow::ControlFlow, environment::Environment, CFResult, Interpreter},
  span::Span,
  token::{Token, TokenType},
};

#[derive(Clone)]
pub enum LoxValue {
  Function(Rc<dyn LoxCallable>),
  Boolean(bool),
  Number(f64),
  String(String),
  Nil,
  Unset,
}

impl LoxValue {
  /// Returns the canonical type name.
  pub fn type_name(&self) -> &'static str {
    use LoxValue::*;
    match self {
      Boolean(_) => "boolean",
      Number(_) => "number",
      String(_) => "string",
      Nil => "nil",
      Function(_) => "<func>",
      Unset => "<unset>",
    }
  }

  /// Converts a `LoxValue` to a Rust bool
  pub fn truth(&self) -> bool {
    use LoxValue::*;
    match self {
      Boolean(inner) => *inner,
      Number(_) | String(_) | Function(_) => true,
      Nil => false,
      Unset => unreachable!("Invalid access of unset variable."),
    }
  }

  /// Checks if two `LoxValue`s are equal. No type coercion is performed so both types must be equal.
  pub fn equals(&self, other: &Self) -> bool {
    use LoxValue::*;
    match (self, other) {
      (Boolean(a), Boolean(b)) => a == b,
      (Number(a), Number(b)) => a == b,
      (String(a), String(b)) => a == b,
      (Nil, Nil) => true,
      _ => false,
    }
  }
}

impl Display for LoxValue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use LoxValue::*;
    match self {
      Function(fun) => Display::fmt(fun, f),
      Boolean(boolean) => Display::fmt(boolean, f),
      Number(number) => {
        if number.floor() == *number {
          // express integers without decimal point
          write!(f, "{:.0}", number)
        } else {
          Display::fmt(number, f)
        }
      }
      String(string) => f.write_str(string),
      Nil => f.write_str("nil"),
      Unset => f.write_str("<unset>"),
    }
  }
}

impl Debug for LoxValue {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use LoxValue::*;
    match self {
      String(s) => write!(f, "\"{}\"", s),
      other => Display::fmt(other, f),
    }
  }
}

#[derive(Debug, Clone)]
pub struct LoxIdent {
  pub id: LoxIdentId,
  pub name: String,
  pub span: Span,
}

// global state:
static LOX_IDENT_ID_SEQ: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct LoxIdentId(usize);

impl LoxIdentId {
  pub fn new() -> Self {
    LoxIdentId(LOX_IDENT_ID_SEQ.fetch_add(1, atomic::Ordering::SeqCst))
  }
}

impl LoxIdent {
  pub fn new(span: Span, name: impl Into<String>) -> Self {
    LoxIdent {
      id: LoxIdentId::new(),
      name: name.into(),
      span,
    }
  }

  /// Creates a new lambda identifier
  pub fn new_lambda(span: Span) -> Self {
    let id = LoxIdentId::new();
    LoxIdent {
      id,
      name: format!("<lambda {}>", id.0),
      span,
    }
  }
}

impl From<Token> for LoxIdent {
  fn from(Token { kind, span }: Token) -> Self {
    match kind {
      TokenType::Identifier(name) => LoxIdent::new(span, name),
      unexpected => unreachable!(
        "Invalid `Token` ({:?}) to `LoxIdent` conversion.",
        unexpected
      ),
    }
  }
}

impl AsRef<str> for LoxIdent {
  fn as_ref(&self) -> &str {
    &self.name
  }
}

impl From<LoxIdent> for String {
  fn from(ident: LoxIdent) -> Self {
    ident.name
  }
}

impl Display for LoxIdent {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.write_str(&self.name)
  }
}

pub trait LoxCallable: Display + Debug {
  fn call(self: Rc<Self>, interpreter: &mut Interpreter, args: &[LoxValue]) -> CFResult<LoxValue>;
  fn arity(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct LoxFunction {
  pub decl: Rc<FunDecl>,
  pub closure: Environment,
  pub is_class_init: bool,
}

impl LoxCallable for LoxFunction {
  fn call(self: Rc<Self>, interpreter: &mut Interpreter, args: &[LoxValue]) -> CFResult<LoxValue> {
    let mut env = Environment::new_enclosed(&self.closure);

    for (param, value) in self.decl.params.iter().zip(args) {
      env.define(param.clone(), value.clone());
    }

    let res = match interpreter.eval_block(&self.decl.body, env) {
      Ok(()) => LoxValue::Nil,
      Err(ControlFlow::Return(val)) => val,
      Err(other) => return Err(other),
    };

    Ok(res)
  }

  fn arity(&self) -> usize {
    self.decl.params.len()
  }
}

impl Display for LoxFunction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<fun {}>", self.decl.name)
  }
}

pub struct NativeFunction {
  pub name: &'static str,
  pub fn_ptr: fn(args: &[LoxValue]) -> CFResult<LoxValue>,
  pub arity: usize,
}

impl LoxCallable for NativeFunction {
  fn call(self: Rc<Self>, _: &mut Interpreter, args: &[LoxValue]) -> CFResult<LoxValue> {
    (self.fn_ptr)(args)
  }

  fn arity(&self) -> usize {
    self.arity
  }
}

impl Display for NativeFunction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<fun (native) {}>", self.name)
  }
}

impl Debug for NativeFunction {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("NativeFunction")
      .field("name", &self.name)
      .field("fn_ptr", &"fn_ptr")
      .field("arity", &self.arity)
      .finish()
  }
}
