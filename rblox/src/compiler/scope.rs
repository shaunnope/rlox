
use std::{cell::RefCell, rc::Rc};

use crate::common::{data::{LoxClosure, LoxFunction, NativeFunction}, Span};

pub struct Local {
  pub name : String,
  pub span: Span,
  pub depth: i32
}

#[derive(Debug, Default)]
pub struct Module {
  pub functions: Vec<Rc<LoxFunction>>,
  pub natives: Vec<Rc<NativeFunction>>,
  pub closures: Vec<Rc<LoxClosure>>,
}

impl Module {
  pub fn new() -> Rc<RefCell<Self>> {
    Rc::new(RefCell::new(Self::default()))
  }
}

pub trait Push<T> {
  fn push(&mut self, obj: T) -> usize;
}

impl Push<LoxFunction> for Module {
  fn push(&mut self, func: LoxFunction) -> usize {
    self.functions.push(Rc::new(func));
    self.functions.len() - 1
  }
}

impl Push<NativeFunction> for Module {
  fn push(&mut self, func: NativeFunction) -> usize {
    self.natives.push(Rc::new(func));
    self.natives.len() - 1
  }
}

impl Push<LoxClosure> for Module {
  fn push(&mut self, func: LoxClosure) -> usize {
    self.closures.push(Rc::new(func));
    self.closures.len() - 1
  }
}