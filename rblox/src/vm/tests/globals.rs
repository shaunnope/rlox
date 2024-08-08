use super::*;

#[test]
fn can_print() {
  let source = "print 1+2;
  print 3*4; print !5.4 == true;";
  let mut vm = VM::new();
  
  if let Err(err) = vm.run(source) {
    eprintln!("{err:?}")
  };
}

#[test]
fn can_def_and_get_globals() {
  let source = "var beverage = \"cafe au lait\";
var breakfast = \"beignets with \" + beverage;
print breakfast;";
  let mut vm = VM::new();
  
  if let Err(err) = vm.run(source) {
    eprintln!("{err:?}")
  };
}

#[test]
fn can_def_get_and_set_globals() {
  let source = "var breakfast = \"beignets\";
var beverage = \"cafe au lait\";
breakfast = \"beignets with \" + beverage;

print breakfast;";
  let mut vm = VM::new();
  
  if let Err(err) = vm.run(source) {
    eprintln!("{err:?}")
  };
}