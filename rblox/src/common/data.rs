use std::{fmt::{Debug, Display}, mem, rc::Rc};

use crate::{
  common::{
    Chunk, 
    error::ErrorLevel,
    Span,
    Value
  },
  compiler::{
    parser::error::ParseError,
    scanner::token::{Token, TokenType}
  }, vm::error::RuntimeError
};

#[derive(Debug, Clone, PartialEq)]
pub enum LoxObject {
  Identifier(String),
  String(String),
  Function(String, usize),
  Native(String, usize),
  Closure(String, usize)
}

impl LoxObject {
  /// Returns the canonical type name.
  pub fn type_name(&self) -> &'static str {
    use LoxObject::*;
    match self {
      Identifier(_) => "<ident>",
      String(_) => "string",
      Function(_, _) | Closure(_, _) => "<func>",
      Native(_, _) => "<native fn>",
      // Class(_) => "<class>",
      // Object(_) => "<instance>",
    }
  }

  pub fn data(&self) -> &String {
    use LoxObject::*;
    match self {
      Identifier(s) | 
      String(s) | 
      Function(s, _) |
      Native(s, _) |
      Closure(s, _)
      => s
    }
  }

  pub fn is_type(&self, other: LoxObject) -> bool {
    mem::discriminant(self) == mem::discriminant(&other)
  }

  pub fn is_callable(&self) -> bool {
    use LoxObject::*;
    match self {
      Function(_, _) | Native(_, _) | Closure(_, _) => true,
      _ => false
    }
  }
}

impl Display for LoxObject {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use LoxObject::*;
    match self {
      Identifier(s) => write!(f, "{s}"),
      String(s) => write!(f, "{s}"),
      Function(name, n) => write!(f, "<fn {name} {n}>"),
      Native(name, _) => write!(f, "<std {name}>"),
      Closure(name, n) => write!(f, "<fn'{name} {n}>"),
    }
  }
}

impl TryFrom<Token> for LoxObject {
  type Error = ParseError;
  fn try_from(value: Token) -> Result<Self, Self::Error> {
    match value.kind {
      TokenType::Identifier(s) => Ok(LoxObject::Identifier(s)),
      _ => Err(ParseError::UnexpectedToken { 
        message: "Expected identifier".into(), 
        offending: value, 
        expected: Some(TokenType::Identifier("<ident>".into()))
      }) 
    }
  }
}

#[derive(PartialEq)]
pub struct LoxFunction {
  pub name: String,
  pub arity: usize,
  pub chunk: Chunk,
  pub upvalues: usize,
}

impl LoxFunction {
  pub fn new(name: &str) -> Self {
    Self {
      name: name.into(),
      arity: 0,
      chunk: Chunk::new(name),
      upvalues: 0
    }
  }
}

impl Debug for LoxFunction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "\n<--- fn {} ({}) --->\n", self.name, self.arity)?;
    write!(f, "{}", self.chunk)
  }
}

pub struct NativeFunction {
  pub name: &'static str,
  pub arity: usize,
  pub fn_ptr: fn(&[Value]) -> Result<Value, RuntimeError>
}

impl NativeFunction {
  pub fn call(&self, args: &[Value], span: Span) -> Result<Value, RuntimeError> {
    if args.len() != self.arity {
      return Err(RuntimeError::UnsupportedType {  
        message: format!(
          "Expected {} arguments, but got {}",
          self.arity,
          args.len()
        ), 
        span, 
        level: ErrorLevel::Error
      })
    }

    (self.fn_ptr)(args)
  }
}

impl Debug for NativeFunction {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "<native {} ({})>", self.name, self.arity)
  }
}

// #[derive(Debug)]
pub struct LoxClosure {
  pub fun: Rc<LoxFunction>,
  pub upvalues: Vec<LoxUpvalue>
}

impl LoxClosure {
  pub fn new(func: Rc<LoxFunction>) -> Self {
    let upvalues = Vec::with_capacity(func.upvalues);
    Self { fun: func.clone(), upvalues }
  }
}

impl From<LoxFunction> for LoxClosure {
  fn from(function: LoxFunction) -> Self {
    let upvalues = Vec::with_capacity(function.upvalues);
    Self { fun: Rc::new(function), upvalues }
  }
}

impl Debug for LoxClosure {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "< fn {} ({}) [{}] >", self.fun.name, self.fun.arity, self.fun.upvalues)
  }
}

#[derive(Debug)]
pub struct LoxUpvalue(pub Rc<Value>);