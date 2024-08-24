
use std::cell::RefCell;

use crate::{
  common::{data::{LoxFunction, LoxObject}, error::ErrorLevel, Chunk, Ins, Span},
  compiler::{
    parser::{
      error::ParseError,
      PResult, Parser, ParserOutcome
    },
    scope::Local
  },
  gc::Module
};

#[cfg(test)]
mod tests;

pub mod scanner;
pub mod parser;

pub mod scope;

pub fn compile(src: &str, module: &mut Module) -> ParserOutcome {
  let parser = Parser::new(src, module);
  parser.parse()
}

pub struct Compiler {
  pub function: LoxFunction,
  pub fun_type: FunctionType,
  pub locals: Vec<Local>,
  scope_depth: i32,
  enclosing: Option<Box<RefCell<Compiler>>>,
  upvalues: Vec<(bool, usize)>,
}

#[derive(PartialEq)]
pub enum FunctionType {
  Function,
  Native,
  Script,
}

impl Compiler {
  const LOCALS_MIN: usize = 128;
  const LOCALS_MAX: usize = 512;
  pub fn new() -> Self {
    Self::build("<script>", FunctionType::Script)
  }

  fn build(name: &str, fun_type: FunctionType) -> Self {
    let mut locals = Vec::with_capacity(Self::LOCALS_MIN);
    locals.push(Local {
      name: name.into(),
      span: Span::new(0,0,0),
      depth: 0,
      captured: false
    });

    Self {
      function: LoxFunction::new(name),
      fun_type,
      locals,
      scope_depth: 0,
      enclosing: None,
      upvalues: Vec::new()
    }
  }

  fn chunk(&mut self) -> &mut Chunk {
    &mut self.function.chunk
  }

  fn begin_scope(&mut self) {
    self.scope_depth += 1;
  }

  fn end_scope(&mut self, span: Span) {
    self.scope_depth -= 1;

    let mut pops = 0;
    while self.locals.len() > 0 && 
    self.locals.last().unwrap().depth > self.scope_depth {
      if self.locals.last().unwrap().captured {
        if pops > 0 {
          self.emit(Ins::PopN(pops), span);
          pops = 0;
        }
        self.emit(Ins::CloseUpval, span);

      } else {
      pops += 1;
      }
      self.locals.pop();
    }

    if pops > 0 {
      self.emit(Ins::PopN(pops), span);
    }
  }

  fn declare_variable(&mut self, ident: &LoxObject, span: Span) -> PResult<()> {
    if self.scope_depth == 0 {
      return Ok(())
    }

    let name = match ident {
      LoxObject::Identifier(name) => name,
      _ => unreachable!()
    };

    if self.locals.len() == 0 {
      self.add_local(name, span)?;
      return Ok(())
    }

    let mut err = None;
    for i in (0..self.locals.len()).rev() {
      let local = &self.locals[i];
      if local.depth != -1 && local.depth < self.scope_depth {
        break;
      }

      if *name == local.name {
        err = Some(ParseError::Error { 
          level: ErrorLevel::Warning, 
          message: format!("Variable `{name}` is already declared in this scope"), 
          span
        });
        break;
      }
    }
    self.add_local(name, span)?;

    match err {
      Some(err) => Err(err),
      None => Ok(())
    }
  }

  fn add_local(&mut self, name: impl Into<String>, span: Span) -> PResult<()> {
    if self.locals.len() == Self::LOCALS_MAX {
      return Err(ParseError::StackOverflow { 
        message: "Too many local variables in function".into(), 
        span 
      })
    }

    self.locals.push(Local {
      name: name.into(),
      span,
      depth: -1,
      captured: false
    });

    Ok(())
  }

  fn mark_init(&mut self) {
    if self.scope_depth == 0 { return };
    let local = self.locals.last_mut().unwrap();
    local.depth = self.scope_depth;
  }

  fn resolve_local(&self, name: &str) -> PResult<Option<usize>> {
    if self.locals.len() == 0 {
      return Ok(None)
    }
    for (i, local) in self.locals.iter().enumerate().rev() {
      if name == local.name {
        if local.depth == -1 {
          return Err(ParseError::Error { 
            level: ErrorLevel::Error, 
            message: format!("Can't read local variable `{}` in its own initializer", local.name), 
            span: local.span
          })
        }
        return Ok(Some(i))
      }
    }
    Ok(None)
  }

  fn resolve_upvalue(&mut self, name: &str, span: Span) -> PResult<Option<usize>> {
    let local = if let Some(enc) = &self.enclosing {
      let mut enc = enc.borrow_mut();
      if let Some(local) = enc.resolve_local(name)? {
        Some((true, local))
      } else if let Some(upv) = enc.resolve_upvalue(name, span)? {
        Some((false, upv))
      } else {
        None
      }
    } else {
      None
    };

    match local {
      Some((is_loc, idx)) => {
        if is_loc {
          if let Some(enc) = &self.enclosing {
            let mut enc = enc.as_ref().borrow_mut();
            let val = enc.locals.get_mut(idx).unwrap();
            val.captured = true;
          } else {
            unreachable!("Enclosing compiler should exist since `is_loc` is true")
          };
        }
        Ok(Some(self.add_upvalue((is_loc, idx), span)?))
      },
      None => Ok(None)
    }
  }

  fn add_upvalue(&mut self, local: (bool, usize), span: Span) -> PResult<usize> {
    let count = self.function.upvalues;

    for (off, pair) in self.upvalues.iter().enumerate() {
      if *pair == local {
        return Ok(off)
      }
    }
    if count == Self::LOCALS_MAX {
      return Err(ParseError::StackOverflow { 
        message: "Too many closure variables in function".into(), 
        span
      })
    }

    self.upvalues.push(local);
    self.function.upvalues += 1;

    Ok(count)
  }

  fn bind(&mut self, enclosing: Compiler) {
    self.enclosing = Some(Box::new(RefCell::new(enclosing)));
  }

  fn unbind(&mut self) -> Compiler {
    let enclosing = match self.enclosing.take() {
      Some(enc) => enc.into_inner(),
      None => unreachable!("`unbind` should always be called after `bind`.")
    };

    enclosing
  }

}

/// Chunk writers
impl Compiler {
  const JUMP_MAX: usize = std::u16::MAX as usize;
  fn emit(&mut self, ins: Ins, span: Span) -> usize {
    let chunk = self.chunk();
    chunk.write(ins, span);
    chunk.len() - 1
  }

  fn patch_jump(&mut self, offset: usize, span: Span) -> PResult<()> {
    let chunk = self.chunk();

    assert!(offset <= chunk.len());
    let jump = chunk.len() - offset - 1;
    if jump > Self::JUMP_MAX {
      return Err(ParseError::InvalidJump { 
        message: "Too much code to jump over".into(), 
        span 
      })
    }

    let ins = match chunk.get(offset).unwrap() {
      (Ins::Jump(_), _) => Ins::Jump(jump as isize),
      (Ins::JumpIfFalse(_), _) => Ins::JumpIfFalse(jump as isize),
      (unexpected, span) => return Err(ParseError::InvalidJump { 
        message: format!("Not a jump instruction. Got {unexpected:?}"),
        span: *span
      })
    };
    chunk.code[offset] = ins;
    Ok(())
  }

  fn emit_loop(&mut self, start: usize, span: Span) -> PResult<usize> {
    let chunk = self.chunk();
    if start >= chunk.len() {
      return Err(ParseError::InvalidJump { 
        message: "Cannot jump ahead when looping".into(),
        span
      })
    };

    let offset = chunk.len() + 1 - start;
    if offset > Self::JUMP_MAX {
      return Err(ParseError::InvalidJump { 
        message: "Loop body too large".into(), 
        span 
      })
    }

    Ok(self.emit(Ins::Jump(-(offset as isize)), span))
  }

}