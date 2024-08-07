
use crate::{
  common::{Chunk, Ins, Span},
  compiler::parser::{Parser, ParserOutcome}
};

#[cfg(test)]
mod tests;

pub mod scanner;
mod parser;

pub fn compile(src: &str) -> ParserOutcome {
  let parser = Parser::new(src);

  if cfg!(test) {
    let mut outcome = parser.parse();
    if let Some(chunk) = outcome.0.last_mut() {
      emit(Ins::Return, Span::new(0, 0, 0), chunk);
    }
    return outcome
  }
  parser.parse()
}

pub fn emit(ins: Ins, span: Span, chunk: &mut Chunk) {
  chunk.write(ins, span);
}