use std::{fmt::Display, iter::Zip, slice::Iter};

use crate::common::{Ins, Span};

#[derive(Debug, PartialEq)]
pub struct Chunk {
  pub name: String,
  pub code: Vec<Ins>,
  spans: Vec<Span>,
  // lines: Vec<(usize, u32)>
}

impl Chunk {
  pub fn new(name: impl Into<String>) -> Self {
    // let mut lines = Vec::new();
    // lines.push((0,0));
    Self {
      name: name.into(),
      code: Vec::new(),
      spans: Vec::new(),
      // lines
    }
  }

  /// Write an instruction to the chunk
  pub fn write(&mut self, ins: Ins, span: Span) {
    self.code.push(ins);
    self.spans.push(span);
  }

  pub fn get(&self, offset: usize) -> Option<(&Ins, &Span)> {
    if offset >= self.len() {
      return None
    }
    Some((&self.code[offset], &self.spans[offset]))
  }

  // /// Get the line of an instruction from the stored run-length encoding
  // fn _get_line(&self, idx: usize) -> u32 {
  //   // Should never panic since only valid indices should be passed into this function
  //   let line = self.lines.binary_search_by(|probe| {
  //     probe.0.cmp(&(idx+1))
  //   }).unwrap();
  //   self.lines[line].1
  // }

  pub fn _iter_zip(&self) -> Zip<Iter<Ins>, Iter<Span>> {
    self.code.iter().zip(self.spans.iter())
  }

  pub fn len(&self) -> usize {
    self.code.len()
  }

}


impl Display for Chunk {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "=== {} ===", self.name)?;
    let mut last_line = 0;
    for (ins, span) in self.code.iter().zip(self.spans.iter()) {
      if last_line != span.2 {
        last_line = span.2;
        write!(f, "{:>3}", last_line)?;
      } else {
        f.write_str("  .")?;
      }
      writeln!(f, " | {ins:?}")?;
    }
    Ok(())
  }
}