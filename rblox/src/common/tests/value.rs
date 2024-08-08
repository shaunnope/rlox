use std::rc::Rc;
use data::LoxObject;


use super::*;

#[test]
fn lox_strings_value_eq() {

  assert_eq!(LoxObject::String("asdf".to_string()), LoxObject::String("asdf".to_string()));

  let a =  Value::Object(Rc::new(LoxObject::String("asdf".to_string())));
  let b =  Value::Object(Rc::new(LoxObject::String("asdf".to_string())));

  assert_eq!(a, b);
}