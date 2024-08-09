use std::{collections::HashMap, rc::Rc};

use crate::common::data::LoxObject;


pub struct MemManager {
  objects: Vec<Rc<LoxObject>>,
  strings: HashMap<String, Rc<LoxObject>>
}

impl MemManager {
  pub fn new() -> Self {
    Self {
      objects: Vec::new(),
      strings: HashMap::new()
    }
  }

  pub fn _alloc_obj(&mut self, obj: LoxObject) -> Rc<LoxObject> {
    let obj = Rc::new(obj);
    self._push(&obj);
    obj
  }

  pub fn _push(&mut self, obj: &Rc<LoxObject>) {
    self.objects.push(obj.clone());
  }

  pub fn add_string(&mut self, str: &str) -> Rc<LoxObject> {
    match self.strings.get(str) {
      Some(obj) => obj.clone(),
      None => {
        let obj = Rc::new(LoxObject::String(str.into()));
        
        self.strings.insert(str.into(), obj.clone());
        self.objects.push(obj.clone());
        
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