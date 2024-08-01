use std::rc::Rc;

use crate::{
  data::{LoxIdent, LoxValue, NativeFunction},
  interpreter::{environment::Environment, CFResult},
  span::Span,
};

pub fn attach(globals: &mut Environment) {
  def_native!(
    globals.clock / 0,
    fn clock(_: &[LoxValue]) -> CFResult<LoxValue> {
      use std::time::{SystemTime, UNIX_EPOCH};
      let start = SystemTime::now();
      let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap().as_secs_f64();
      Ok(LoxValue::Number(since_the_epoch))
    }
  );
}

macro_rules! def_native {
  ($globals:ident . $name:ident / $arity:expr  , $fn:item) => {
    $fn
    $globals.define(
      LoxIdent::new(Span::new(0, 0), stringify!($name)),
      LoxValue::Function(Rc::new(NativeFunction {
        name: stringify!($name),
        fn_ptr: $name,
        arity: $arity
      })),
    );
  };
}

use def_native;
