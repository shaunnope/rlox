use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
  common::{
    data::{LoxClosure, LoxObject}, error::{ErrorLevel, ErrorType, LoxError, LoxResult}, 
    Ins, Span, Value
  }, 
  compiler::{compile, scope::{Module, Push}, FunctionType},
  gc::mmap::MemManager,
  vm::error::RuntimeError
};

#[cfg(test)]
use crate::common::{Chunk, data::LoxFunction};

#[cfg(test)]
mod tests;

pub mod error;
pub mod native;

struct CallFrame {
  function: Rc<LoxClosure>,
  ip: usize,
  /// start of VM stack
  start: usize, 
}

impl Display for CallFrame {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      let (_, span) = self.function.fun.chunk.get(self.ip - 1).unwrap();
      write!(f, "[line {}] in {}; at position {}", span.2, self.function.fun.name, span)?;

      Ok(())
  }
}

pub struct VM {
  frames: Vec<CallFrame>,
  stack: Vec<Value>,
  globals: HashMap<String, Value>,
  objects: MemManager,
  span: Span,
  module: Rc<RefCell<Module>>
}

impl VM {
  pub fn run(&mut self, src: &str) -> LoxResult<ErrorType> {
    let compile_errors = compile(src, self.module.clone());

    if compile_errors.len() > 0 {
      // report errors and exit
      for err in compile_errors {
        err.report();
      }
      return Err(ErrorType::CompileError)
    }

    if cfg!(debug_assertions) {
      println!("{:?}", self.module);
    }
    
    let main = self.module.clone().borrow_mut().functions.last().unwrap().clone();

    self.frames.push(CallFrame { 
      function: Rc::new(LoxClosure::new(main)),
      ip: 0, 
      start: 0
    });

    match self.interpret() {
      Err(err) => {
        err.report();
        self.stack_trace();
        Err(ErrorType::RuntimeError)
      },
      Ok(_) => Ok(())
    }
  }

  pub fn interpret(&mut self) -> LoxResult<RuntimeError> {
    use Ins::*;
    use Value as V;

    loop {
      let (mut ip, inst, span) = match self.advance() {
        None => break,
        Some(res) => res
      };

      // if cfg!(features = "debug-step") {
        if cfg!(debug_assertions) {
        display_instr(&self.stack, &inst);
      }
      let mut jumped = false;

      match inst {
        Constant(n) => self.push(n.clone())?,
        True => self.push(Value::Boolean(true))?,
        False => self.push(Value::Boolean(false))?,
        Nil => self.push(Value::Nil)?,

        Negate => {
          let val = self.pop();
          match val {
            V::Number(_) => self.push(-val)?,
            unexpected => return Err(
              RuntimeError::UnsupportedType {
                level: ErrorLevel::Error,
                message: format!(
                  "Bad type for unary `-` operator: `{}`",
                  unexpected.type_name()
                ),
                span,
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
              span,
            })
          };
          self.push(out)?;        
        },
        Subtract => bin_num_op!(self, -),
        Multiply => bin_num_op!(self, *),
        Divide => bin_num_op!(self, /), // TODO:  Raise ZeroDivision error

        Equal => {
          let a = self.pop();
          let b = self.pop();
          self.push(Value::Boolean(a.equals(&b)))?;
        }
        Greater => bin_cmp_op!(self, >),
        Less => bin_cmp_op!(self, <),

        Not => {
          let val = self.pop();
          self.push(Value::Boolean(!val))?
        },

        Print => {
          println!("{}", self.pop())
        }
        Pop => { self.pop(); },
        PopN(n) => { 
          for _ in 0..n {
            self.pop(); 
          }
        },

        DefGlobal(name) => {
          let val = self.peek(0).unwrap().to_owned();
          self.globals.insert(name.to_owned(), val);
          self.pop();
        }
        GetGlobal(name) => {
          match self.globals.get(&name) {
            Some(val) => {
              self.push(val.clone())?;
            },
            None => return Err(RuntimeError::UndefinedVariable { 
              name: name.into(),
              span 
            })
          }
        }
        SetGlobal(name) => {
          if !self.globals.contains_key(&name) {
            return Err(RuntimeError::UndefinedVariable { 
              name: name.into(), 
              span
            })
          }

          let val = self.peek(0).unwrap().to_owned();
          self.globals.insert(name.into(), val);
        }

        GetLocal(slot) => {
          let val = self.get(slot).clone();
          self.push(val)?;
        },
        SetLocal(slot) => {
          let val = self.peek(0).unwrap().clone();
          self.set(slot, val);
        }

        Call(args) => {
          self.call_value(args)?;
        },

        Closure(n) => {
          let func = self.module.clone().borrow_mut()
            .functions.get(n).unwrap().clone();
          let name = func.name.clone();
          let closure = LoxClosure::new(func);

          let offset = self.module.borrow_mut().push(closure);
          self.push(Value::Object(Rc::new(LoxObject::Closure(name, offset))))?;
        }

        Jump(offset) => {
          ip = ((ip as isize) + offset) as usize;
          jumped = true;
        }
        JumpIfFalse(offset) => {
          if !self.peek(0).unwrap().truth() {
            ip = ((ip as isize) + offset) as usize;
            jumped = true;
          }
        }

        Return => {
          let result = self.pop();
          let frame = self.frames.pop().unwrap();
          if self.frames.len() == 0 {
            return Ok(())
          }

          self.pop_to(frame.start);
          self.push(result)?;

        },
        // _ => {}
      }
      
      if jumped { self.update(ip); }
    }
    Ok(())
  }

  fn call_value(&mut self, args: usize) -> LoxResult<RuntimeError> {
    use Value::Object;
    use LoxObject as L;
    use FunctionType as F;

    let callee = self.peek(args).unwrap();
    let (kind, idx) = match callee {
      Object(obj) if obj.is_callable() => {
        match &**obj {
          L::Function(_, _) => unreachable!("Functions should be wrapped as closures."),
          L::Native(_, idx) => {
            (F::Native, *idx)
          },
          L::Closure(_, idx) => {
            (F::Function, *idx)
          }
          _ => unreachable!()
        }
      },
      unexpected => return Err(
        RuntimeError::UnsupportedType { 
          message: format!("Can only call functions and classes. Got `{}`", unexpected.type_name()), 
          span: self.span, 
          level: ErrorLevel::Error 
        }
      )
    };

    match kind {
      F::Function => {
        let function = self.module.clone().borrow_mut().closures.get(idx).unwrap().clone();

        self.call(function, args)?;
      },
      F::Native => {
        let native = self.module.clone().borrow_mut().natives.get(idx).unwrap().clone();
        
        let start = self.stack.len()-args-1;
        let args = &self.stack[start..self.stack.len()-1];
        
        let res = native.call(args, self.span)?;
        self.pop_to(start);
        self.push(res)?;
      }
      _ => unreachable!()
    };

    Ok(())
  }

  fn call(&mut self, closure: Rc<LoxClosure>, args: usize) -> LoxResult<RuntimeError> {
    if args != closure.fun.arity {
      return Err(RuntimeError::UnsupportedType {  
        message: format!(
          "Expected {} arguments, but got {}",
          closure.fun.arity,
          args
        ), 
        span: self.span, 
        level: ErrorLevel::Error
      })
    }

    if self.frames.len() == Self::FRAMES_MAX {
      return Err(RuntimeError::StackOverflow(self.span))
    }

    let start = self.stack.len()-args-1;
    self.frames.push(CallFrame {
      function: closure.clone(),
      ip: 0,
      start
    });
    Ok(())
  }

}

/// Stack operations
impl VM {
  const FRAMES_MAX: usize = 64;
  const STACK_MAX: usize = Self::FRAMES_MAX * std::u8::MAX as usize;
  const STACK_MIN: usize = 64;
  pub fn new() -> Self {
    let mut vm = Self {
      frames: Vec::new(),
      stack: Vec::with_capacity(Self::STACK_MIN),
      globals: HashMap::new(),
      objects: MemManager::new(),
      span: Span::new(0, 0, 0),
      module: Module::new()
    };

    vm.stack.push(Value::Object(Rc::new(LoxObject::Function("<main>".into(), 0))));

    attach(&mut vm);
    vm
  }

  /// Push value onto stack
  fn push(&mut self, value: Value) -> LoxResult<RuntimeError> {
    if self.stack.len() == Self::STACK_MAX {
      return Err(RuntimeError::StackOverflow(self.span))
    }
    self.stack.push(value);
    Ok(())
  }

  /// Pop value from stack.
  fn pop(&mut self) -> Value {
    // should not panic due to correctness of parser
    self.stack.pop().unwrap()
  }

  /// Pop from stack until a target size
  fn pop_to(&mut self, offset: usize) {
    while self.stack.len() > offset {
      self.pop();
    }
  }

  /// Peek at value a relative distance from the top of stack.
  fn peek(&mut self, distance: usize) -> Option<&Value> {
    if self.stack.len()-1 < distance {
      None
    } else {
      Some(&self.stack[self.stack.len()-1-distance])
    }
  }

  /// Get value from stack relative to start of top frame
  fn get(&mut self, slot: usize) -> &Value {
    let frame = self.frames.last().unwrap();
    self.stack.get(frame.start+slot).unwrap()
  }

  /// Set value in stack relative to start of top frame
  fn set(&mut self, slot: usize, value: Value) {
    let frame = self.frames.last().unwrap();
    let val = self.stack.get_mut(frame.start+slot).unwrap();
    *val = value;
  }

  /// Advance ip
  fn advance(&mut self) -> Option<(usize, Ins, Span)> {
    let frame = self.frames.last_mut().unwrap();
    let chunk = &frame.function.fun.chunk;

    match chunk.get(frame.ip) {
      None => None,
      Some((ins, span)) => {
        frame.ip += 1;
        self.span = *span;
        Some((frame.ip, ins.clone(), *span))
      }
    }
  }

  /// Update ip
  fn update(&mut self, ip: usize) {
    let frame = self.frames.last_mut().unwrap();
    frame.ip = ip
  }

  fn stack_trace(&mut self) {
    for frame in self.frames.iter().rev() {
      eprintln!("{}", frame)
    }
  }

  #[cfg(test)]
  fn add_chunk(&mut self, chunk: Chunk) {
    let function = Rc::new(LoxClosure::from(
      LoxFunction {
        name: chunk.name.clone(),
        arity: 0,
        chunk
      }
    ));

    self.frames.push(CallFrame {
      function,
      ip: 0,
      start: 0
    })
  }

}

// #[allow(dead_code)]
fn display_instr(stack: &[Value], inst: &Ins) {
  print!("[ ");
  for slot in stack.iter() {
    print!("{slot:?}, ");
  }
  println!("]\n{:?}", inst);
}

macro_rules! bin_num_op {
  ($self:expr, $op:tt) => {{
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
          span: $self.span,
        }) 
    };
    $self.push(out)?;
  }}
}
use bin_num_op;

macro_rules! bin_cmp_op {
  ($self:expr, $op:tt) => {{
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
          span: $self.span,
        }) 
    };
    $self.push(out)?;
  }}
}
use bin_cmp_op;
use native::attach;