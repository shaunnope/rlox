use super::*;

#[test]
fn correct_return() {
  let mut chunk = Chunk::new("return");
  chunk.write(Ins::Return, 1);
  chunk.write(Ins::Return, 1);
  chunk.write(Ins::Return, 2);

  assert_eq!(chunk.to_string(), 
  "=== return ===
    1 | OP_RETURN
    . | OP_RETURN
    2 | OP_RETURN\n");
}

#[test]
fn correct_constant() {
  use Value::Number;
  let mut chunk = Chunk::new("constant");
  chunk.write(Ins::Constant(Number(1.0)), 2);
  chunk.write(Ins::Constant(Number(1.2)), 4);
  chunk.write(Ins::Constant(Number(2.13)), 4);

  assert_eq!(chunk.to_string(), 
  "=== constant ===
    2 | OP_CONST       1
    4 | OP_CONST       1.2
    . | OP_CONST       2.13\n");
}