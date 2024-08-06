use super::*;

#[cfg(test)]
mod arith;

#[test]
fn correct_arith() {
  let mut vm = VM::new();
  let mut chunk = Chunk::new("test chunk");
  chunk.write(Ins::Constant(Value::Number(1.2)), 1);
  chunk.write(Ins::Constant(Value::Number(3.4)), 2);
  chunk.write(Ins::Add, 2);
  chunk.write(Ins::Constant(Value::Number(5.6)), 2);
  chunk.write(Ins::Divide, 3);
  chunk.write(Ins::Negate, 3);
  chunk.write(Ins::Return, 3);
  let _ = vm.interpret(chunk);
}

#[test]
fn process_arith() {
  let source = "1+2-3*-4/(5-6)";
  let mut vm = VM::new();

  if let Err(err) = vm.run(source) {
    eprintln!("{err:?}")
  };
}

#[test]
fn challenge_17_1() {
  let source = "(-1+2)*3--4";
  let mut vm = VM::new();

  if let Err(err) = vm.run(source) {
    eprintln!("{err:?}")
  };
}