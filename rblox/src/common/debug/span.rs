use std::{
  cmp::{max, min},
  fmt::{self, Display},
  ops::Range,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
/// Represents a string fragment.
/// The bounds are over its byte representation.
pub struct Span(pub usize, pub usize, pub u32);

impl Span {
  /// Create a new span.
  pub fn new(lo: usize, hi: usize, line: u32) -> Span {
    Span(min(lo, hi), max(lo, hi), line)
  }

  #[cfg(test)]
  pub fn dummy(line: u32) -> Span {
    Span::new(0,0,line)
  }

  /// Create a new span encompassing `self` and `other`.
  pub fn to(&self, other: Span) -> Span {
    Span::new(min(self.0, other.0), max(self.1, other.1), min(self.2, other.2))
  }

  /// Check if the span contains the given position.
  pub fn contains_p(&self, position: usize) -> bool {
    self.0 <= position && position < self.1
  }

  /// Modify the given span. Panic if new bounds are invalid.
  pub fn updated(&self, lo: isize, hi: isize) -> Span {
    let lo = self.0 as isize + lo;
    let hi = self.1 as isize + hi;
    assert!(lo >= 0, "New lower bound can't be negative.");
    assert!(lo <= hi, "Lower bound can not pass the higher.");
    Span::new(lo as _, hi as _, self.2)
  }

  /// Return the span range.
  pub fn range(&self) -> Range<usize> {
    Range {
      start: self.0,
      end: self.1,
    }
  }
}

impl Display for Span {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    if (self.1 - self.0) <= 1 {
      write!(f, "{}", self.0)
    } else {
      write!(f, "{}..{}", self.0, self.1)
    }
  }
}
