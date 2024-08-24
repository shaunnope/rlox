use std::{cell::RefCell, collections::HashMap, fmt::Display, rc::Rc};

use crate::{
  common::{
    data::{LoxClosure, LoxObject, LoxUpvalue}, error::{ErrorLevel, ErrorType, LoxError, LoxResult}, 
    Ins, Span, Value
  }, 
  compiler::{compile, FunctionType},
  gc::{
    data::Push,
    MemManager, Module
  },
  vm::error::RuntimeError
};

#[cfg(test)]
use crate::common::{Chunk, data::LoxFunction};

#[cfg(test)]
mod tests;

pub mod error;
pub mod native;

struct CallFrame {
  function: Rc<RefCell<LoxClosure>>,
  ip: usize,
  /// start of VM stack
  start: usize, 
}

impl Display for CallFrame {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      let func = self.function.borrow();
      let (_, span) = func.fun.chunk.get(self.ip - 1).unwrap();
      write!(f, "[line {}] in {}; at position {}", span.2, func.fun.name, span)?;

      Ok(())
  }
}

pub struct VM {
  frames: Vec<CallFrame>,
  stack: Vec<Value>,
  globals: HashMap<String, Value>,
  objects: MemManager,
  span: Span,
  module: Module
}

impl VM {
  pub fn run(&mut self, src: &str) -> LoxResult<ErrorType> {
    let compile_errors = compile(src, &mut self.module);

    if compile_errors.len() > 0 {
      // report errors and exit
      for err in compile_errors {
        err.report();
      }
      return Err(ErrorType::CompileError)
    }

    if cfg!(debug_assertions) {
      println!("{}", self.module);
    }
    
    let main = self.module.functions.last().unwrap().clone().unwrap();

    self.frames.push(CallFrame { 
      function: Rc::new(RefCell::new(LoxClosure::new(main))),
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

      // if cfg!(feature = "dbg-step") {
      // if cfg!(debug_assertions) {
      //   display_instr(&self.stack, &inst);
      // }
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
        Divide => {
          let b = self.pop();
          let a = self.pop();

          use Value::*;
          let out = match (a, b) {
            (Number(a), Number(b)) => {
              if b == 0.0 {
                let warn = RuntimeError::ZeroDivision(self.span);
                warn.report();
              }
              Number(a / b)
            },
            (a, b) => return Err(RuntimeError::UnsupportedType {
              level: ErrorLevel::Error,
              message: format!(
                "Binary `/` operator can only operate over two numbers. \
                Got types `{}` and `{}`",
                a.type_name(),
                b.type_name()
              ),
              span,
            })
          };
          self.push(out)?;          
        }, // TODO:  Raise ZeroDivision error

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

        GetUpval(slot) => {
          use LoxUpvalue::*;
          let val = self.get_upvalue(slot);
          let val = match &*val.borrow() {
            Open(pos) => self.stack.get(*pos).unwrap().clone(),
            Closed(val) => val.copy()
          };

          self.push(val)?;
        },
        SetUpval(slot) => {
          let val = self.peek(0).unwrap().copy();
          self.set_upvalue(slot, val);
        }
        CloseUpval => {
          self.close_upvals(self.frames.last().unwrap().start, self.stack.len()-1);
          self.pop();
        }


        Call(args) => {
          self.call_value(args)?;
        },

        Closure(n, upvals) => {
          let closure = LoxClosure::new(
            self.module.functions.get(n).unwrap().clone().unwrap()
          );
          let n = self.module.push(closure);

          let closure = self.module.closures.last().unwrap().clone();
          let name = closure.borrow().fun.name.clone();
          
          for (is_local, idx) in upvals.iter() {
            let upval = if *is_local {
              self.capture_upval(*idx)?
            } else {
              self.get_upvalue(*idx)
            };

            closure.borrow_mut().upvalues.push(upval);
          }

          self.push(Value::Object(Rc::new(LoxObject::Closure(name, n))))?;
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
          self.close_upvals(frame.start, frame.start);
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
        let function = self.module.closures.get(idx).unwrap();

        self.call(function, args)?;
      },
      F::Native => {
        let native = self.module.natives.get(idx).unwrap().clone();
        
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

  fn call(&mut self, closure: Rc<RefCell<LoxClosure>>, args: usize) -> LoxResult<RuntimeError> {
    if args != closure.borrow().fun.arity {
      return Err(RuntimeError::UnsupportedType {  
        message: format!(
          "Expected {} arguments, but got {}",
          closure.borrow().fun.arity,
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
      objects: MemManager::default(),
      span: Span::new(0, 0, 0),
      module: Module::default()
    };

    vm.stack.push(Value::Object(Rc::new(LoxObject::Function("<script>".into(), 0))));

    attach(&mut vm);
    vm
  }

  /// Push value onto stack
  fn push(&mut self, value: Value) -> LoxResult<RuntimeError> {
    if self.stack.len() == Self::STACK_MAX {
      return Err(RuntimeError::StackOverflow(self.span))
    }

    let value = if let Value::Object(obj) = &value {
      if let LoxObject::String(str) = &**obj {
        Value::Object(self.objects.add_string(str))
      } else {
        value
      }
    } else {
      value
    };

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

  /// Get upvalue in top frame
  fn get_upvalue(&mut self, slot: usize) -> Rc<RefCell<LoxUpvalue>> {
    let frame = self.frames.last().unwrap();
    frame.function.borrow().upvalues.get(slot).unwrap().clone()
  }

  /// Set indexed upvalue to a value
  fn set_upvalue(&mut self, slot: usize, value: Value) {
    let fun = self.frames.last_mut().unwrap().function.borrow_mut();
    
    // if Open, update stack, else, update upval.
    let mut upval = fun.upvalues.get(slot).unwrap().borrow_mut();
    let updated = match &*upval {
      LoxUpvalue::Open(pos) => {
        let val = self.stack.get_mut(*pos).unwrap();
        *val = value;
        return;
      },
      LoxUpvalue::Closed(_) => LoxUpvalue::from(value)
    };

    *upval = updated

  }

  /// Capture local variable as an upvalue.
  /// 
  /// If stack slot has already been captured as an upvalue, return the reference to that upvalue.
  /// Otherwise, create a new upvalue.
  fn capture_upval(&mut self, idx: usize) -> Result<Rc<RefCell<LoxUpvalue>>, RuntimeError> {
    let slot = self.frames.last().unwrap().start + idx;

    for upval in self.module.upvals.iter() {
      match &*upval.borrow() {
        LoxUpvalue::Open(pos) => {
          if *pos == slot { return Ok(upval.clone())}
          if *pos < slot { break; }
        }
        LoxUpvalue::Closed(_) => continue,
      }
    };

    let upval = Rc::new(RefCell::new(LoxUpvalue::from(slot)));
    // TODO: insert in sorted order?
    self.module.upvals.push(upval.clone());

    Ok(upval)
  }

  fn close_upvals(&mut self, start: usize, last: usize) {
    assert!(last < self.stack.len());
    assert!(start <= last);
    let last = last - start;

    for upval in self.module.upvals.iter_mut() {
      let closed = match &*upval.borrow() {
        LoxUpvalue::Open(slot) if *slot >= last => {
          let val = self.stack.get(*slot).unwrap().clone();
          LoxUpvalue::from(val)
        }
        _ => continue
      };
      let mut upval = upval.borrow_mut();
      *upval = closed;
    }

  }

  /// Advance ip
  fn advance(&mut self) -> Option<(usize, Ins, Span)> {
    let frame = self.frames.last_mut().unwrap();
    let chunk = &frame.function.borrow().fun.chunk;

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
    let function = Rc::new(RefCell::new(
      LoxClosure::from(
        LoxFunction {
          name: chunk.name.clone(),
          arity: 0,
          chunk,
          upvalues: 0
        }
      )
    ));

    self.frames.push(CallFrame {
      function,
      ip: 0,
      start: 0
    })
  }

}

#[allow(dead_code)]
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