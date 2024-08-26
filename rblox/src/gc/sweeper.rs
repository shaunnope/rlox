use std::rc::Rc;

use super::data::{Allocated, Push};


/// Mark objects that have been checked by the Gc by storing Rc handles
#[derive(Default)]
pub(crate) struct Sweeper {
  objects: Vec<Rc<dyn Allocated>>
}

impl Push<Rc<dyn Allocated>> for Sweeper {
  fn push(&mut self, obj: Rc<dyn Allocated>) -> usize {
    self.objects.push(obj);
    self.objects.len() - 1
  }
}
