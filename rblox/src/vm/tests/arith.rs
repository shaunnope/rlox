use super::*;

#[test]
fn challenge_1_1() {
  let mut vm = VM::new();
  let mut chunk = Chunk::new("challenge 1.1");
  chunk.write(Ins::Constant(Value::Number(1.0)), 1);
  chunk.write(Ins::Constant(Value::Number(2.0)), 2);
  chunk.write(Ins::Multiply, 2);
  chunk.write(Ins::Constant(Value::Number(3.0)), 2);
  chunk.write(Ins::Add, 3);
  chunk.write(Ins::Return, 3);
  vm.interpret(chunk);
}

#[test]
fn challenge_1_2() {
  let mut vm = VM::new();
  let mut chunk = Chunk::new("challenge 1.2");
  chunk.write(Ins::Constant(Value::Number(1.0)), 1);
  chunk.write(Ins::Constant(Value::Number(2.0)), 2);
  chunk.write(Ins::Constant(Value::Number(3.0)), 2);
  chunk.write(Ins::Multiply, 2);
  chunk.write(Ins::Add, 3);
  chunk.write(Ins::Return, 3);
  vm.interpret(chunk);
}

#[test]
fn challenge_1_3() {
  let mut vm = VM::new();
  let mut chunk = Chunk::new("challenge 1.3");
  chunk.write(Ins::Constant(Value::Number(3.0)), 1);
  chunk.write(Ins::Constant(Value::Number(2.0)), 2);
  chunk.write(Ins::Subtract, 2);
  chunk.write(Ins::Constant(Value::Number(1.0)), 2);
  chunk.write(Ins::Subtract, 3);
  chunk.write(Ins::Return, 3);
  vm.interpret(chunk);
}

#[test]
fn challenge_1_4() {
  let mut vm = VM::new();
  let mut chunk = Chunk::new("challenge 1.4");
  chunk.write(Ins::Constant(Value::Number(1.0)), 1);
  chunk.write(Ins::Constant(Value::Number(2.0)), 2);
  chunk.write(Ins::Constant(Value::Number(3.0)), 2);
  chunk.write(Ins::Multiply, 2);
  chunk.write(Ins::Add, 2);
  chunk.write(Ins::Constant(Value::Number(4.0)), 2);
  chunk.write(Ins::Constant(Value::Number(5.0)), 2);
  chunk.write(Ins::Negate, 2);
  chunk.write(Ins::Divide, 2);
  chunk.write(Ins::Subtract, 3);
  chunk.write(Ins::Return, 3);
  vm.interpret(chunk);
}

#[test]
fn challenge_2_1() {
  let mut vm = VM::new();
  let mut chunk = Chunk::new("challenge 2.1 No Negate");
  chunk.write(Ins::Constant(Value::Number(4.0)), 1);
  chunk.write(Ins::Constant(Value::Number(3.0)), 2);
  chunk.write(Ins::Constant(Value::Number(0.0)), 2);
  chunk.write(Ins::Constant(Value::Number(2.0)), 2);
  chunk.write(Ins::Subtract, 2);
  chunk.write(Ins::Multiply, 2);
  chunk.write(Ins::Subtract, 2);
  chunk.write(Ins::Return, 3);
  vm.interpret(chunk);
}

#[test]
fn challenge_2_2() {
  let mut vm = VM::new();
  let mut chunk = Chunk::new("challenge 2.1 No Subtract");
  chunk.write(Ins::Constant(Value::Number(4.0)), 1);
  chunk.write(Ins::Constant(Value::Number(3.0)), 2);
  chunk.write(Ins::Constant(Value::Number(2.0)), 2);
  chunk.write(Ins::Negate, 2);
  chunk.write(Ins::Multiply, 2);
  chunk.write(Ins::Negate, 2);
  chunk.write(Ins::Add, 2);
  chunk.write(Ins::Return, 3);
  vm.interpret(chunk);
}