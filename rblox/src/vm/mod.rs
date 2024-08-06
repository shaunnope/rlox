use crate::{
  common::{
    error::{LoxResult, ErrorType}, 
    Chunk, 
    Ins, 
    Value
  }, 
  compiler::compile
};

#[cfg(test)]
mod tests;

pub struct VM {
  stack: Vec<Value>
}


impl VM {
  pub fn run(&mut self, src: &str) -> LoxResult<ErrorType> {
    let (chunks, compile_errors) = compile(src);

    if compile_errors.len() > 0 {
      // report errors and exit
      for err in compile_errors {
        eprintln!("{}", err)
      }
      return Err(ErrorType::CompileError)
    }

    let chunk = chunks.last().unwrap().to_owned();

    if cfg!(debug_assertions) {
      println!("{}", chunk);
    }

    match self.interpret(chunk) {
      Err(_) => Err(ErrorType::CompileError),
      Ok(_) => Ok(())
    }
  }
  pub fn interpret(&mut self, chunk: Chunk) -> LoxResult<()> {
    use Ins::*;
    for inst in chunk.code {
      // if cfg!(debug_assertions) {
      //   display_instr(&self.stack, &inst);
      // }

      match inst {
        Constant(n) => self.push(n),
        Negate => {
          let val = self.pop();
          self.stack.push(-val);
        },
        Add => arith_bin!(self, +),
        Subtract => arith_bin!(self, -),
        Multiply => arith_bin!(self, *),
        Divide => arith_bin!(self, /),
        Return => {
          if let Some(val) = self.stack.pop() {
            println!("{:?}", val);
            return Ok(())
          } else {
            return Err(())
          }          
        },
        // _ => {}
      }
    }
    Ok(())
  }
}

impl VM {
  pub fn new() -> Self {
    Self {
      stack: Vec::new()
    }
  }

  fn push(&mut self, value: Value) {
    self.stack.push(value);
  }

  fn pop(&mut self) -> Value {
    self.stack.pop().unwrap()
  }
}

fn display_instr(stack: &[Value], inst: &Ins) {
  print!("          [ ");
  for slot in stack.iter() {
    print!("{slot:?}, ");
  }
  println!("]\n{:?}\n", inst);

}

macro_rules! arith_bin {
  ($self:expr, $op:tt) => {{
    let b = $self.pop();
    let a = $self.pop();
    use Value::*;
    let out = match (a, b) {
      (Number(a), Number(b)) => Number(a $op b),
    };
    $self.push(out);
  }}
}
use arith_bin;