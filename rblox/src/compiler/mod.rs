
use crate::{
  common::{Chunk, Ins, Span},
  compiler::parser::{Parser, ParserOutcome}
};

#[cfg(test)]
mod tests;

pub mod scanner;
pub mod parser;

pub fn compile(src: &str) -> ParserOutcome {
  let parser = Parser::new(src);

  parser.parse()
}

pub fn emit(ins: Ins, span: Span, chunk: &mut Chunk) {
  chunk.write(ins, span);
}