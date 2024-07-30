use std::{
  error::Error,
  fmt::{self, Display},
};

use crate::span::Span;

#[derive(Debug, Clone)]
pub enum RuntimeError {
  UnsupportedType { message: String, span: Span },

  // UndefinedVariable { ident: LoxIdent },
  // UndefinedProperty { ident: LoxIdent },
  ZeroDivision { span: Span },
}

impl Display for RuntimeError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use RuntimeError::*;
    match self {
      UnsupportedType { message, span } => {
        write!(f, "{}; at position {}", message, span)
      }

      ZeroDivision { span } => {
        write!(f, "Can not divide by zero; at position {}", span)
      }
    }
  }
}

impl RuntimeError {
  /// Returns the span that caused the error.
  pub fn primary_span(&self) -> Span {
    use RuntimeError::*;
    match self {
      UnsupportedType { span, .. } | ZeroDivision { span } => *span,
    }
  }
}

impl Error for RuntimeError {}
