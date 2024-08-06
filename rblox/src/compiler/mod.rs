
use crate::{
  common::{Chunk, Ins},
  compiler::parser::{Parser, ParserOutcome}
};

#[cfg(test)]
mod tests;

pub mod scanner;
mod parser;

pub fn compile(src: &str) -> ParserOutcome {
  let parser = Parser::new(src);

  parser.parse()  
}

pub fn emit(ins: Ins, line: u32, chunk: &mut Chunk) {
  chunk.write(ins, line);
}