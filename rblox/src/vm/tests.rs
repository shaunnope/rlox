use super::*;

use crate::common::Span;

mod challenges;
mod variables;
mod sequence;
mod functions;

#[test]
fn correct_arith() {
  let mut vm = VM::new();
  let mut chunk = Chunk::new("test chunk");
  chunk.write(Ins::Constant(Value::Number(1.2)), Span::dummy(1));
  chunk.write(Ins::Constant(Value::Number(3.4)), Span::dummy(2));
  chunk.write(Ins::Add, Span::dummy(2));
  chunk.write(Ins::Constant(Value::Number(5.6)), Span::dummy(2));
  chunk.write(Ins::Divide, Span::dummy(3));
  chunk.write(Ins::Negate, Span::dummy(3));
  chunk.write(Ins::Return, Span::dummy(3));
  vm.add_chunk(chunk);
  let _ = vm.interpret();
}

#[test]
fn process_arith() {
  let source = "print 1+2-3*-4/(5-6);";
  let mut vm = VM::new();

  if let Err(err) = vm.run(source) {
    eprintln!("{err:?}")
  };
}

#[test]
fn process_literals() {
  let mut vm = VM::new();

  if let Err(err) = vm.run("true;") {
    eprintln!("{err:?}")
  };

  if let Err(err) = vm.run("false;") {
    eprintln!("{err:?}")
  };

  if let Err(err) = vm.run("nil;") {
    eprintln!("{err:?}")
  };
}

#[test]
fn process_types() {
  let source = "!(5 - 4 > 3 * 2 == !nil)";
  let mut vm = VM::new();

  if let Err(err) = vm.run(source) {
    eprintln!("{err:?}")
  };
}

#[test]
fn concat_strings() {
  let source = "print \"st\" + \"ri\" + \"ng\";";
  let mut vm = VM::new();

  if let Err(err) = vm.run(source) {
    eprintln!("{err:?}")
  };
}
