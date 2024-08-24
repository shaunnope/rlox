use std::cell::{Ref, RefCell, RefMut};


pub trait Push<T> {
  fn push(&mut self, obj: T) -> usize;
}

pub trait Allocated {

}

pub struct AlRefCell<T: Allocated>(RefCell<T>);

impl<T: Allocated> AlRefCell<T> {
  pub fn new(value: T) -> Self {
    Self(RefCell::new(value))
  }

  pub fn borrow(&self) -> Ref<T> {
    self.0.borrow()
  }

  pub fn borrow_mut(&self) -> RefMut<T> {
    self.0.borrow_mut()
  }
}


impl<T: Allocated> Allocated for AlRefCell<T> {}

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

pub(crate) struct IterMut<'a, T: Clone> {
  data: &'a mut [Option<T>],
  curr: usize
}

impl<'a, T: Clone> IterMut<'a, T> {
  pub(crate) fn new(data: &'a mut [Option<T>]) -> Self {
    Self {
      data,
      curr: 0
    }
  }
}

impl<'a, T: Clone> Iterator for IterMut<'a, T> {
  type Item = &'a mut T;
  fn next(&mut self) -> Option<Self::Item> {
    loop {
      if self.curr >= self.data.len() {
        return None
      }

       // Unsafe block is necessary to create a mutable reference from self.slice.
       let item = unsafe { &mut *self.data.as_mut_ptr().add(self.curr) };
       self.curr += 1;

      match item {
        None => {},
        Some(_) => return item.as_mut()
      };
    }; 

  }
}

