use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
  data::{LoxIdent, LoxValue},
  interpreter::error::RuntimeError,
};

#[derive(Debug, Default)]
struct EnvironmentInner {
  enclosing: Option<Environment>,
  locals: HashMap<String, LoxValue>,
}

#[derive(Debug, Clone, Default)]
pub struct Environment {
  inner: Rc<RefCell<EnvironmentInner>>,
}

impl Environment {
  /// Creates a new `Environment` with one scope (i.e. the global scope).
  pub fn new() -> Self {
    Default::default()
  }

  /// Returns a new environment that is enclosed by the given env
  pub fn new_enclosed(enclosing: &Self) -> Self {
    Self {
      inner: Rc::new(RefCell::new(EnvironmentInner {
        enclosing: Some(enclosing.clone()),
        locals: HashMap::new(),
      })),
    }
  }

  /// Returns the enclosed environment.
  pub fn enclosed(&self) -> Option<Environment> {
    self.inner.borrow().enclosing.clone()
  }

  /// Defines a variable
  pub fn define(&mut self, name: LoxIdent, value: LoxValue) {
    self.inner.borrow_mut().locals.insert(name.into(), value);
  }

  /// Assigns a variable
  pub fn assign(&mut self, ident: &LoxIdent, value: LoxValue) -> Result<LoxValue, RuntimeError> {
    let mut inner = self.inner.borrow_mut();
    match inner.locals.get_mut(&ident.name) {
      Some(var) => {
        *var = value.clone();
        Ok(value)
      }
      None => match &mut inner.enclosing {
        // recursive assignment up scope stack
        Some(enclosing) => enclosing.assign(ident, value),
        // no match in global scope = undefined
        None => Err(RuntimeError::UndefinedVariable {
          ident: ident.clone(),
        }),
      },
    }
  }

  /// Assigns to a variable in a distant scope.
  pub fn assign_at(&mut self, dist: usize, ident: &LoxIdent, value: LoxValue) -> LoxValue {
    // This should never panic due to the semantic verifications that the resolver performs.
    *self
      .ancestor(dist)
      .inner
      .borrow_mut()
      .locals
      .get_mut(&ident.name)
      .unwrap() = value.clone();
    value
  }

  /// Reads a variable.
  pub fn read(&self, ident: &LoxIdent) -> Result<LoxValue, RuntimeError> {
    let inner = self.inner.borrow();
    match inner.locals.get(&ident.name) {
      Some(LoxValue::Unset) => Err(RuntimeError::UnsetVariable {
        ident: ident.clone(),
      }),
      Some(var) => Ok(var.clone()),
      None => match &inner.enclosing {
        Some(enclosing) => enclosing.read(ident),
        None => Err(RuntimeError::UndefinedVariable {
          ident: ident.clone(),
        }),
      },
    }
  }

  /// Reads a variable in a distant scope.
  pub fn read_at(&self, dist: usize, ident: impl AsRef<str>) -> LoxValue {
    self
      .ancestor(dist)
      .inner
      .borrow()
      .locals
      .get(ident.as_ref())
      .unwrap()
      .clone()
  }

  fn ancestor(&self, dist: usize) -> Environment {
    let mut curr = self.clone();
    for _ in 0..dist {
      let maybe_enclosing = curr.inner.borrow().enclosing.clone();
      curr = maybe_enclosing.unwrap();
    }
    curr
  }
}
