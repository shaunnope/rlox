use std::{any::Any, cell::RefCell, collections::{BinaryHeap, HashMap}, fmt::{Debug, Display}, rc::Rc};

use crate::{
  common::data::{
    LoxClosure, LoxFunction, LoxObject, LoxUpvalue, NativeFunction
  }, 
  gc::data::{
    Allocated, Iter, IterMut, Push
  }
};

pub type Gcc<T> = Option<Rc<T>>;

type Container<T> = Option<Rc<T>>; 


#[derive(Debug)]
pub struct Gc<T>
{
  data: Vec<Container<T>>,
  free: BinaryHeap<usize>
}

impl<T> Default for Gc<T> {
  fn default() -> Self {
    Self {
      data: Vec::new(),
      free: BinaryHeap::new()
    }
  }
}

impl<T> Gc<T> {
  pub fn iter(&self) -> Iter<'_, Rc<T>> {
    Iter::new(&self.data)
  }

  pub fn iter_mut(&mut self) -> IterMut<'_, Rc<T>> {
    IterMut::new(self.data.as_mut_slice())
  }

  pub fn last(&self) -> Option<&Rc<T>> {
    self.iter().last()
  }

  pub fn get(&self, idx: usize) -> Option<Rc<T>> {
    if idx >= self.data.len() {
      return None
    }
    self.data.get(idx).unwrap().clone()
  }

  /// Free up allocations
  pub fn free(&mut self) -> bool {
    if cfg!(feature = "dbg-gc") {
      println!("--- gc begin")
    }

    let mut freed = false;
    for (i, val) in self.data.iter_mut().enumerate() {
      let free = if let Some(inner) = val { 
        Self::check(inner)
      } else {
        false
      };

      if free {
        if cfg!(feature = "dbg-gc") {
          println!("{i}");
        }
        // *val = None;
        // self.free.push(i);
        freed = true;
      }
    }

    if cfg!(feature = "dbg-gc") {
      println!("--- gc end")
    }
    freed
  }

}


impl<T> Gc<T> {
  fn check(obj: &mut Rc<T>) -> bool {
    Rc::strong_count(obj) == 1
  }
  
}

impl<T> Push<Rc<T>> for Gc<T> {
  fn push(&mut self, obj: Rc<T>) -> usize {
    if cfg!(debug_assertions) {
      // "stress test" GC by running it at every allocation
      self.free();
    }

    let item = Some(obj.clone());
    if self.free.peek() == None {
      self.data.push(item);

      if cfg!(feature = "dbg-gc") {
        println!("Pushed obj to end")
      }

      return self.data.len() - 1;
    }

    let pos = *self.free.peek().unwrap();
    let val = self.data.get_mut(pos).unwrap();
    *val = item;  

    self.free.pop(); // unfree after allocation 

    if cfg!(feature = "dbg-gc") {
      println!("Inserted obj at {pos}")
    }

    pos
  }

}

impl<T> Push<T> for Gc<T> {
  fn push(&mut self, obj: T) -> usize {
    self.push(Rc::new(obj))
  }

}

#[derive(Default)]
pub struct Module {
  pub functions: Vec<Gcc<LoxFunction>>,
  pub natives: Vec<Rc<NativeFunction>>,
  pub closures: Gc<RefCell<LoxClosure>>,
  pub upvals: Gc<RefCell<LoxUpvalue>>,
  objects: Vec<Rc<LoxObject>>,
  strings: HashMap<String, Rc<LoxObject>>
}

impl Module {
  pub fn new() -> Rc<RefCell<Self>> {
    Rc::new(RefCell::new(Self::default()))
  }

  pub fn alloc_obj(&mut self, obj: Rc<LoxObject>) -> Rc<LoxObject> {
    if let LoxObject::String(str) = &*obj {
      self.add_string(str)
    } else {
      self.push(obj.clone());
      obj
    }
  }

  pub fn  add_string(&mut self, str: &str) -> Rc<LoxObject> {
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

  pub fn free(&mut self) {
    self.upvals.free();
  }

}

impl Display for Module {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    const PAD: usize = 31;
    writeln!(f, "{:=^1$}", "| FUNCTIONS |", PAD)?;
    for func in self.functions.iter() {
      match func {
        None => writeln!(f, "{:?}", func)?,
        Some(inner) => writeln!(f, "{:?}", inner)?,
      }
    }
    writeln!(f, "{:=^1$}", "| NATIVES |", PAD)?;
    for func in self.natives.iter() {
      write!(f, "{:?}, ", func)?;
    }
    writeln!(f, "\n{:=^1$}", "", PAD)?;
    Ok(())
  }
}

impl Push<LoxObject> for Module {
  fn push(&mut self, obj: LoxObject) -> usize {
    self.push(Rc::new(obj))
  }
}

impl Push<Rc<LoxObject>> for Module {
  fn push(&mut self, obj: Rc<LoxObject>) -> usize {
    self.objects.push(obj);
    self.objects.len() - 1
  }
}

impl Push<LoxFunction> for Module {
  fn push(&mut self, func: LoxFunction) -> usize {
    self.functions.push(Some(Rc::new(func)));
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
    self.closures.push(RefCell::new(func))
  }

}
