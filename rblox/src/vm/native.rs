use std::{cell::RefCell, rc::Rc};

use crate::{
  common::{data::{LoxObject, NativeFunction}, Value},
  gc::{
    data::Push,
    Module
  },
  vm::{error::RuntimeError, VM}
};

/// Define native functions as globals to vm
pub fn attach(vm: &mut VM) {
  let mut module = Module::default();

  def_native!(
    vm.module.clock / 0,
    fn clock(_: &[Value]) -> Result<Value, RuntimeError> {
      use std::time::{SystemTime, UNIX_EPOCH};
      let start = SystemTime::now();
      let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
      Ok(Value::Number(since_the_epoch))
    }
  );

  vm.module = Rc::new(RefCell::new(module));
}

macro_rules! def_native {
  ($vm:ident . $module:ident . $name:ident / $arity:expr  , $fn:item) => {
    $fn
    let name = stringify!($name);
    let n = $module.push(NativeFunction {
      name,
      fn_ptr: $name,
      arity: $arity
    });

    $vm.globals.insert(
      name.into(),
      Value::Object(Rc::new(
        LoxObject::Native(name.into(), n)
      ))
    );
  };
}

use def_native;