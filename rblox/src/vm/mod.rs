use std::collections::HashMap;

use crate::{
  common::{
    data::LoxObject,
    error::{LoxError, ErrorLevel, ErrorType, LoxResult}, 
    Chunk, 
    Ins, 
    Value
  }, 
  compiler::compile,
  gc::mmap::MemManager,
  vm::error::RuntimeError
};

#[cfg(test)]
mod tests;

pub mod error;

pub struct VM {
  stack: Vec<Value>,
  globals: HashMap<String, Value>,
  objects: MemManager
}


impl VM {
  pub fn run(&mut self, src: &str) -> LoxResult<ErrorType> {
    let (chunks, compile_errors) = compile(src);

    if compile_errors.len() > 0 {
      // report errors and exit
      for err in compile_errors {
        err.report();
      }
      return Err(ErrorType::CompileError)
    }

    let chunk = chunks.last().unwrap().to_owned();

    if cfg!(debug_assertions) {
      println!("{}", chunk);
    }

    match self.interpret(chunk) {
      Err(err) => {
        err.report();
        Err(ErrorType::CompileError)
      },
      Ok(_) => Ok(())
    }
  }
  pub fn interpret(&mut self, chunk: Chunk) -> LoxResult<RuntimeError> {
    use Ins::*;
    use Value as V;
    for (inst, span ) in chunk.iter_zip() {
      // if cfg!(debug_assertions) {
      //   display_instr(&self.stack, &inst);
      // }

      match inst {
        Constant(n) => self.push(n.clone()),
        True => self.push(Value::Boolean(true)),
        False => self.push(Value::Boolean(false)),
        Nil => self.push(Value::Nil),

        Negate => {
          let val = self.pop();
          match val {
            V::Number(_) => self.push(-val),
            unexpected => return Err(
              RuntimeError::UnsupportedType {
                level: ErrorLevel::Error,
                message: format!(
                  "Bad type for unary `-` operator: `{}`",
                  unexpected.type_name()
                ),
                span: *span,
              },
            ),
          };
        },
        Add => {
          let b = self.pop();
          let a = self.pop();

          use Value::*;
          use LoxObject as L;
          let out = match (a, b) {
            (Number(a), Number(b)) => Number(a + b),
            (Object(a), b) if a.is_type(L::String("".into()))
            => {
              match &*a {
                L::String(a) => {
                  let obj = self.objects.add_string(
                    &(a.to_owned() + &b.to_string())
                  );
                  Object(obj)
                },
                _ => unreachable!()
              }
            },
            (a, b) => return Err(RuntimeError::UnsupportedType {
              level: ErrorLevel::Error,
              message: format!(
                "Binary `+` operator can only operate over two numbers or strings. \
                Got types `{}` and `{}`",
                a.type_name(),
                b.type_name()
              ),
              span: *span,
            })
          };
          self.push(out);        
        },
        Subtract => bin_num_op!(self, -, *span),
        Multiply => bin_num_op!(self, *, *span),
        Divide => bin_num_op!(self, /, *span), // TODO:  Raise ZeroDivision error

        Equal => {
          let a = self.pop();
          let b = self.pop();
          self.push(Value::Boolean(a.equals(&b)));
        }
        Greater => bin_cmp_op!(self, >, *span),
        Less => bin_cmp_op!(self, <, *span),

        Not => {
          let val = self.pop();
          self.push(Value::Boolean(!val))
        },

        Print => {
          println!("{}", self.pop())
        }
        Pop => { self.pop(); },

        DefGlobal(name) => {
          let val = self.peek(0).unwrap().to_owned();
          self.globals.insert(name.to_owned(), val);
          self.pop();
        }
        GetGlobal(name) => {
          match self.globals.get(name) {
            Some(val) => {
              self.push(val.clone());
            },
            None => return Err(RuntimeError::UndefinedVariable { 
              name: name.into(),
              span: *span 
            })
          }
        }
        SetGlobal(name) => {
          if !self.globals.contains_key(name) {
            return Err(RuntimeError::UndefinedVariable { 
              name: name.into(), 
              span: *span, 
            })
          }

          let val = self.peek(0).unwrap().to_owned();
          self.globals.insert(name.into(), val);
        }

        Return => {},
        // _ => {}
      }
    }
    Ok(())
  }
}

impl VM {
  pub fn new() -> Self {
    Self {
      stack: Vec::new(),
      globals: HashMap::new(),
      objects: MemManager::new()
    }
  }

  /// Push value onto stack
  fn push(&mut self, value: Value) {
    // if let Value::Object(obj) = &value {
    //   self.objects.push(obj);
    // }
    self.stack.push(value);
  }

  /// Pop value from stack.
  fn pop(&mut self) -> Value {
    // should not panic due to correctness of parser
    self.stack.pop().unwrap()
  }

  /// Peek at value a relative distance from the top of stack.
  fn peek(&mut self, distance: usize) -> Option<&Value> {
    if self.stack.len()-1 < distance {
      None
    } else {
      Some(&self.stack[self.stack.len()-1-distance])
    }
  }
}

fn display_instr(stack: &[Value], inst: &Ins) {
  print!("[ ");
  for slot in stack.iter() {
    print!("{slot:?}, ");
  }
  println!("]\n{:?}", inst);

}

macro_rules! bin_num_op {
  ($self:expr, $op:tt, $span:expr) => {{
    let b = $self.pop();
    let a = $self.pop();
    use Value::*;
    let out = match (a, b) {
      (Number(a), Number(b)) => Number(a $op b),
      (a, b) => return Err(
        RuntimeError::UnsupportedType {
          level: ErrorLevel::Error,
          message: format!(
            "Binary `{}` operator can only operate over two numbers. \
            Got types `{}` and `{}`",
            stringify!($op),
            a.type_name(),
            b.type_name()
          ),
          span: $span,
        }) 
    };
    $self.push(out);
  }}
}
use bin_num_op;

macro_rules! bin_cmp_op {
  ($self:expr, $op:tt, $span:expr) => {{
    let b = $self.pop();
    let a = $self.pop();
    use Value::*;
    let out = match (a, b) {
      (Number(a), Number(b)) => Boolean(a $op b),
      (a, b) => return Err(
        RuntimeError::UnsupportedType {
          level: ErrorLevel::Error,
          message: format!(
            "Binary `{}` operator can only compare two numbers. \
            Got types `{}` and `{}`",
            stringify!($op),
            a.type_name(),
            b.type_name()
          ),
          span: $span,
        }) 
    };
    $self.push(out);
  }}
}
use bin_cmp_op;