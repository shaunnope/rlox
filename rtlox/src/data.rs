use std::{
  cell::RefCell, collections::HashMap, fmt::{self, Debug, Display}, rc::Rc, sync::atomic::{self, AtomicUsize}
};

use crate::{
  ast::stmt::FunDecl,
  interpreter::{control_flow::ControlFlow, environment::Environment, error::RuntimeError, CFResult, Interpreter},
  span::Span,
  token::{Token, TokenType},
};

#[derive(Clone)]
pub enum LoxValue {
  Function(Rc<dyn LoxCallable>),
  Class(Rc<LoxClass>),
  Object(Rc<LoxInstance>),
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
      Class(_) => "<class>",
      Object(_) => "<instance>",
      Unset => "<unset>",
    }
  }

  /// Converts a `LoxValue` to a Rust bool
  pub fn truth(&self) -> bool {
    use LoxValue::*;
    match self {
      Boolean(inner) => *inner,
      Number(_) | String(_) | Function(_) | 
      Class(_) | Object(_) => true,
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
      Class(class) => Display::fmt(class, f),
      Object(instance) => Display::fmt(instance, f),
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

impl LoxFunction {
  pub fn bind(&self, instance: &Rc<LoxInstance>) -> Rc<Self> {
    let mut env = Environment::new_enclosed(&self.closure);
    env.define("this", LoxValue::Object(instance.clone()));
    Rc::new(LoxFunction {
        decl: self.decl.clone(),
        closure: env,
        is_class_init: self.is_class_init,
    })
  }
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

    if self.is_class_init {
      Ok(self.closure.read_at(0, "this"))
    } else {
      Ok(res)
    }
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

#[derive(Debug, Clone)]
pub struct LoxClass {
  pub name: LoxIdent,
  pub methods: HashMap<String, Rc<LoxFunction>>,
}

impl LoxClass {
  pub fn get_method(&self, ident: impl AsRef<str>) -> Option<Rc<LoxFunction>> {
    self.methods
        .get(ident.as_ref())
        .cloned()
        .or_else(||None)
  }
}

impl Display for LoxClass {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<class {}>", self.name)
  }
}

impl LoxCallable for LoxClass {
  fn call(
    self: Rc<Self>, 
    interpreter: &mut Interpreter, 
    args: &[LoxValue]
  ) -> CFResult<LoxValue> {
    let instance = Rc::new(LoxInstance {
      name: LoxIdent::new(
        Span::new(0,0), 
        self.name.name.clone()
      ),
      constructor: self,
      properties: RefCell::new(HashMap::new()),
    });
    if let Some(init) = instance.get_bound_method("init") {
      init.call(interpreter, args)?;
    }

    Ok(LoxValue::Object(instance))
  }

  fn arity(&self) -> usize {
    if let Some(init) = self.get_method("init") {
      init.arity()
    } else {
      0
    }
  }
}

#[derive(Debug, Clone)]
pub struct LoxInstance {
  pub constructor: Rc<LoxClass>,
  pub name: LoxIdent,
  properties: RefCell<HashMap<String, LoxValue>>
}

impl LoxInstance {
  pub fn get(
    self: &Rc<Self>, 
    ident: &LoxIdent
  ) -> Result<LoxValue, RuntimeError> {
    // Fields looked up before properties (methods)
    // => Shadows methods
    if let Some(value) = self.properties.borrow().get(&ident.name) {
      return Ok(value.clone());
    }

    if let Some(method) = self.get_bound_method(ident) {
      return Ok(LoxValue::Function(method));
    }

    Err(RuntimeError::UndefinedProperty {
      ident: ident.clone(),
    })
  }

  pub fn set(&self, ident: &LoxIdent, value: LoxValue) {
    self.properties
      .borrow_mut()
      .insert(ident.name.clone(), value);
  }

  pub fn get_bound_method(self: &Rc<Self>, ident: impl AsRef<str>) -> Option<Rc<LoxFunction>> {
    self.constructor
      .get_method(ident)
      .map(|unbound| unbound.bind(self))
  }
}

impl Display for LoxInstance {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "<instance {}>", self.name)
  }
}