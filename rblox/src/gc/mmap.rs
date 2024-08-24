use std::{collections::HashMap, rc::Rc};

use crate::common::data::LoxObject;

#[derive(Default)]
pub struct MemManager {
  objects: Vec<Rc<LoxObject>>,
  strings: HashMap<String, Rc<LoxObject>>
}

impl MemManager {
  pub fn alloc_obj(&mut self, obj: Rc<LoxObject>) -> Rc<LoxObject> {
    if let LoxObject::String(str) = &*obj {
      self.add_string(str)
    } else {
      self.push(obj.clone());
      obj
    }
  }

  pub fn push(&mut self, obj: Rc<LoxObject>) {
    self.objects.push(obj);
  }

  pub fn add_string(&mut self, str: &str) -> Rc<LoxObject> {
    match self.strings.get(str) {
      Some(obj) => obj.clone(),
      None => {
        let obj = Rc::new(LoxObject::String(str.into()));
        
        self.strings.insert(str.into(), obj.clone());
        self.push(obj.clone());
        
        obj
      }
    }
  }

  pub fn take_string(&mut self, str: &str) -> Rc<LoxObject> {
    match self.strings.get(str) {
      Some(_) => {
        self.strings.remove(str).unwrap()
      },
      None => Rc::new(LoxObject::String(str.into()))
    }
  }

  pub fn find_string(&mut self, str: &str) -> Option<Rc<LoxObject>> {
    self.strings.get(str).cloned()
  }
}