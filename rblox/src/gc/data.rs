use std::{cell::{Ref, RefCell as StdRefCell, RefMut}, fmt::Debug};

use super::sweeper::Sweeper;


pub trait Push<T> {
  fn push(&mut self, obj: T) -> usize;
}

/// Trait for objects that can be gc'd
pub trait Allocated: Debug {
  /// Returns `true` if object can be freed. Else, `false`
  fn check(&self, sweeper: &mut Sweeper) -> bool;
}

#[allow(unused_variables)]
pub trait Allocatable: Debug {
  fn check(&self, sweeper: &mut Sweeper) -> bool {
    false
  }
}

// Wrapper to impl Allocated trait
pub struct RefCell<T: Allocatable + ?Sized>(StdRefCell<T>);

impl<T: Allocatable> RefCell<T> {
  pub fn new(value: T) -> Self {
    Self(StdRefCell::new(value))
  }

  pub fn borrow(&self) -> Ref<T> {
    self.0.borrow()
  }

  pub fn borrow_mut(&self) -> RefMut<T> {
    self.0.borrow_mut()
  }
}

impl<T: Allocatable> Debug for RefCell<T> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      self.0.borrow().fmt(f)
  }
}


impl<T: Allocatable> Allocated for RefCell<T> {
  fn check(&self, sweeper: &mut Sweeper) -> bool {
    self.0.borrow().check(sweeper)
  }
}

pub(crate) struct Iter<'a, T> {
  data: &'a [Option<T>],
  curr: usize,
  last: usize
}

impl<'a, T> Iter<'a, T> {
  pub(crate) fn new(data: &'a [Option<T>]) -> Self {
    Self {
      data,
      curr: 0,
      last: data.len()
    }
  }

}

impl<'a, T> Iterator for Iter<'a, T> {
  type Item = &'a T;
  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if self.curr >= self.last {
        return None
      }
      self.curr += 1;

      if let Some(val) = self.data.get(self.curr-1).unwrap() {
        return Some(val)
      }
    }
  }

}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
  fn next_back(&mut self) -> Option<Self::Item> {
    loop {
      if self.last <= self.curr {
        return None
      }
      self.last -= 1;

      if let Some(val) = self.data.get(self.last).unwrap() {
        return Some(val)
      }
    };
  }
}
