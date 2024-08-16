use super::*;

#[test]
fn division_by_zero() {
  let source = "
  print 1/0;
  print 2/(-3+3);
  print 1/0 + 2;
  ";

  let mut vm = VM::new();
  if let Err(err) = vm.run(source) {
    eprintln!("{err:?}")
  };
}