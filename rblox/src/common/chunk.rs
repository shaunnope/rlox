use std::fmt::Display;

use crate::common::OpCode;

#[derive(Debug, Clone, PartialEq)]
pub struct Chunk {
  pub name: String,
  code: Vec<OpCode>,
  lines: Vec<(usize, u32)>
}

impl Chunk  {
  pub fn new(name: impl Into<String>) -> Self {
    let mut lines = Vec::new();
    lines.push((0,0));
    Self {
      name: name.into(),
      code: Vec::new(),
      lines
    }
  }

  /// Write an instruction to the chunk
  pub fn write(&mut self, ins: OpCode, line: u32) {
    self.code.push(ins);
    let last = self.lines.last_mut().unwrap();
    let count = last.0;
    if line == last.1 {
      *last = (last.0 + 1, last.1);
    } else {
      self.lines.push((count + 1, line));
    }
  }

  /// Get the line of an instruction from the stored run-length encoding
  fn get_line(&self, idx: usize) -> u32 {
    // Should never panic since only valid indices should be passed into this function
    let line = self.lines.binary_search_by(|probe| {
      probe.0.cmp(&(idx+1))
    }).unwrap();
    self.lines[line].1
  }
}


impl Display for Chunk {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    writeln!(f, "=== {} ===", self.name)?;
    let mut line_idx = 0;
    for (idx, ins) in self.code.iter().enumerate() {
      if idx+1 > self.lines[line_idx].0 {
        line_idx += 1;
        write!(f, "{:>5}", self.lines[line_idx].1)?;
      } else {
        f.write_str("    .")?;
      }
      writeln!(f, " | {ins}")?;
    }
    Ok(())
  }
}