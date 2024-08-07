use std::fmt::{self, Display};

#[derive(Debug, Clone, PartialEq)]
pub enum ScanError {
  UnexpectedChar(char),

  UnterminatedString,
  UnterminatedComment,

  InvalidNumberLiteral,
}

impl Display for ScanError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use ScanError::*;
    match self {
      UnexpectedChar(char) => write!(f, "Unexpected character `{}`", char),
      UnterminatedString => f.write_str("Unterminated string"),
      UnterminatedComment => f.write_str("Unterminated block comment"),
      InvalidNumberLiteral => f.write_str("Unparseable number literal"),
    }
  }
}

impl ScanError {
  /// Checks if the error allows REPL continuation (aka. "..." prompt).
  pub fn allows_continuation(&self) -> bool {
    matches!(self, ScanError::UnterminatedString)
  }
}
