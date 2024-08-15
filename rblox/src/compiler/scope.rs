
use std::{cell::RefCell, fmt::Display, rc::Rc};

use crate::common::{data::{LoxClosure, LoxFunction, LoxUpvalue, NativeFunction}, Span};

pub struct Local {
  pub name : String,
  pub span: Span,
  pub depth: i32,
  pub captured: bool
}

#[derive(Debug, Default)]
pub struct Module {
  pub functions: Vec<Rc<LoxFunction>>,
  pub natives: Vec<Rc<NativeFunction>>,
  pub closures: Vec<Rc<RefCell<LoxClosure>>>,
  pub upvals: Vec<Rc<RefCell<LoxUpvalue>>>
}

impl Module {
  pub fn new() -> Rc<RefCell<Self>> {
    Rc::new(RefCell::new(Self::default()))
  }
}

impl Display for Module {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    const PAD: usize = 31;
    writeln!(f, "{:=^1$}", "| FUNCTIONS |", PAD)?;
    for func in self.functions.iter() {
      writeln!(f, "{:?}", func)?;
    }
    writeln!(f, "{:=^1$}", "| NATIVES |", PAD)?;
    for func in self.natives.iter() {
      write!(f, "{:?}, ", func)?;
    }
    writeln!(f, "\n\n{:=^1$}\n", "| CLOSURES |", PAD)?;
    for func in self.closures.iter() {
      writeln!(f, "{:?}", func.borrow())?;
    }

    writeln!(f, "\n{:=^1$}", "", PAD)?;
    Ok(())
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
    self.closures.push(Rc::new(RefCell::new(func)));
    self.closures.len() - 1
  }
}