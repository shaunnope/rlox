use parser::error::ParseError;

use crate::{
  common::{data::LoxObject, error::ErrorLevel, Chunk, Ins, Span},
  compiler::{
    parser::{PResult, Parser, ParserOutcome},
    scope::Local
  }
};

#[cfg(test)]
mod tests;

pub mod scanner;
pub mod parser;

mod scope;

pub fn compile(src: &str) -> ParserOutcome {
  let compiler = Compiler::new();
  let parser = Parser::new(src, compiler);

  parser.parse()
}

pub fn emit(ins: Ins, span: Span, chunk: &mut Chunk) -> usize {
  chunk.write(ins, span);
  chunk.len() - 1
}

pub fn patch_jump(offset: usize, span: Span, chunk: &mut Chunk) -> PResult<()> {
  assert!(offset <= chunk.len());
  let jump = chunk.len() - offset - 1;
  if jump as u16 > std::u16::MAX {
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

pub fn emit_loop(start: usize, span: Span, chunk: &mut Chunk) -> PResult<usize> {
  if start >= chunk.len() {
    return Err(ParseError::InvalidJump { 
      message: "Cannot jump ahead when looping".into(),
      span
    })
  };

  let offset = chunk.len() + 1 - start;
  if offset as u16 > std::u16::MAX {
    return Err(ParseError::InvalidJump { 
      message: "Loop body too large".into(), 
      span 
    })
  }

  Ok(emit(Ins::Jump(-(offset as isize)), span, chunk))
}

pub struct Compiler {
  pub locals: Vec<Local>,
  scope_depth: i32
}

impl Compiler {
  const MAX_SIZE: usize = 512;
  pub fn new() -> Self {
    Self {
      locals: Vec::with_capacity(Self::MAX_SIZE),
      scope_depth: 0
    }
  }

  fn begin_scope(&mut self) {
    self.scope_depth += 1;
  }

  fn end_scope(&mut self) -> usize {
    self.scope_depth -= 1;

    let mut pops = 0;
    while self.locals.len() > 0 && 
    self.locals.last().unwrap().depth > self.scope_depth {
      pops += 1;
      self.locals.pop();
    }
    pops
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
    if self.locals.len() == self.locals.capacity() {
      return Err(ParseError::Error { 
        level: ErrorLevel::Error, 
        message: "Too many local variables in function".into(), 
        span 
      })
    }

    self.locals.push(Local {
      name: name.into(),
      span,
      depth: -1
    });

    Ok(())
  }

  fn mark_init(&mut self) {
    let local = self.locals.last_mut().unwrap();
    local.depth = self.scope_depth;
  }

  fn resolve_local(&self, name: &str) -> PResult<Option<usize>> {
    if self.locals.len() == 0 {
      return Ok(None)
    }
    for i in (0..self.locals.len()).rev() {
      let local = &self.locals[i];
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
}
