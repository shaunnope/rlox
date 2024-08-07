use std::rc::Rc;

use crate::common::data::LoxObject;


pub struct MemManager {
  pub objects: Vec<Rc<LoxObject>>
}

impl MemManager {
  pub fn new() -> Self {
    Self {
      objects: Vec::new()
    }
  }

  pub fn _alloc_obj(&mut self, obj: LoxObject) -> Rc<LoxObject> {
    let obj = Rc::new(obj);
    self.push(&obj);
    obj
  }

  pub fn push(&mut self, obj: &Rc<LoxObject>) {
    self.objects.push(obj.clone());
  }
}