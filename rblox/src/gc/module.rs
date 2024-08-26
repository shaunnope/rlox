use std::{collections::{BinaryHeap, HashMap}, fmt::{Debug, Display}, rc::Rc};

use crate::{
  common::data::{
    LoxClosure, LoxFunction, LoxObject, LoxUpvalue, NativeFunction
  }, 
  gc::{
    data::{
      Allocated, Iter, Push, RefCell
    },
    sweeper::Sweeper
  }
};

type Container<T> = Option<Rc<T>>; 


#[derive(Debug)]
pub struct Gc<T: Allocated>
{
  data: Vec<Container<T>>,
  free: BinaryHeap<usize>
}

impl<T: Allocated> Default for Gc<T> {
  fn default() -> Self {
    Self {
      data: Vec::new(),
      free: BinaryHeap::new()
    }
  }
}

impl<T: Allocated + 'static> Gc<T> {
  pub fn iter(&self) -> Iter<'_, Rc<T>> {
    Iter::new(&self.data)
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
      println!("\x1b[2m--- gc begin")
    }

    let mut freed = false;
    let mut sweeper = Sweeper::default();
    for (i, val) in self.data.iter_mut().enumerate() {
      let free = if let Some(inner) = val { 
        Self::check(inner, &mut sweeper)
      } else {
        false
      };

      if free {
        if cfg!(feature = "dbg-gc") {
          val.clone().inspect(|inner| println!("Freed {inner:?}"));
        }
        *val = None;
        self.free.push(i);
        freed = true;
        
      }
    }

    if cfg!(feature = "dbg-gc") {
      println!("--- gc end\x1b[0m")
    }
    freed
  }

  /// Check if object can be freed
  fn check(obj: &mut Rc<T>, sweeper: &mut Sweeper) -> bool {
    if Rc::strong_count(obj) > 1 {
      return false
    }
    sweeper.push(obj.clone());
    obj.check(sweeper)
  }
  
}

impl<T: Allocated + 'static> Push<Rc<T>> for Gc<T> {
  fn push(&mut self, obj: Rc<T>) -> usize {
    if cfg!(debug_assertions) {
      // "stress test" GC by running it at every allocation
      self.free();
    }

    let item = Some(obj.clone());
    if self.free.peek() == None {
      self.data.push(item);

      if cfg!(feature = "dbg-gc") {
        println!("Pushed to end: {obj:?}")
      }

      return self.data.len() - 1;
    }

    let pos = *self.free.peek().unwrap();
    let val = self.data.get_mut(pos).unwrap();
    *val = item;  

    self.free.pop(); // unfree after allocation 

    if cfg!(feature = "dbg-gc") {
      println!("Inserted at {pos}: {obj:?}")
    }

    pos
  }

}

impl<T: Allocated + 'static> Push<T> for Gc<T> {
  fn push(&mut self, obj: T) -> usize {
    self.push(Rc::new(obj))
  }

}

#[derive(Default)]
pub struct Module {
  pub functions: Gc<LoxFunction>,
  pub natives: Vec<Rc<NativeFunction>>,
  pub closures: Gc<LoxClosure>,
  pub upvals: Gc<RefCell<LoxUpvalue>>,
  objects: Vec<Rc<LoxObject>>,
  strings: HashMap<String, Rc<LoxObject>>
}

impl Module {
  pub fn alloc_obj(&mut self, obj: Rc<LoxObject>) -> Rc<LoxObject> {
    if let LoxObject::String(str) = &*obj {
      self.add_string(str)
    } else {
      self.push(obj.clone());
      obj
    }
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

  pub fn free(&mut self) {
    self.upvals.free();
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
    self.functions.push(func)
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
    self.closures.push(func)
  }

}

impl Push<RefCell<LoxUpvalue>> for Module {
  fn push(&mut self, value: RefCell<LoxUpvalue>) -> usize {
    self.upvals.push(value)
  }

}