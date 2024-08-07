use std::{
  // error::Error,
  fmt::{self, Display},
};

use crate::{
  common::{
    error::{Error, ErrorLevel, ErrorType}, 
    Span,
  }, 
  compiler::scanner::{
    error::ScanError,
    token::{Token, TokenType}
  }
};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
  Error {
    level: ErrorLevel,
    message: String,
    span: Span,
  },

  ScanError {
    error: ScanError,
    span: Span,
  },

  UnexpectedToken {
    message: String,
    offending: Token,
    expected: Option<TokenType>,
  },

  DetectedLambda,
}

impl Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use ParseError::*;
    match self {
      Error { message, span , ..} => {
        write!(f, "{}; at position {}", message, span)
      }

      ScanError { error, span } => {
        write!(f, "{}; at position {}", error, span)
      }

      UnexpectedToken {
        message, offending, ..
      } => {
        write!(
          f,
          "{}; unexpected token `{}`; at position {}",
          message, offending, offending.span
        )?;
        // if let Some(expected) = expected {
        //     write!(f, "\nInstead expected token of kind `{}`", expected)?;
        // }
        Ok(())
      }

      DetectedLambda => unreachable!(),
    }
  }
}

impl Error for ParseError {
  fn get_level(&self) -> ErrorLevel {
    ErrorLevel::Error
  }

  fn get_type(&self) -> ErrorType {
    ErrorType::CompileError
  }

  fn get_span(&self) -> Span {
    use ParseError::*;
      match self {
        Error {span, ..} | ScanError {span, ..} => *span,
        UnexpectedToken {offending, ..} => offending.span,
        DetectedLambda => Span::new(0, 0, 0),
      }
  }
}

impl ParseError {
  /// Returns the span that caused the error.
  pub fn primary_span(&self) -> Span {
    use ParseError::*;
    match self {
      Error { span, .. } | ScanError { span, .. } => *span,
      UnexpectedToken { offending, .. } => offending.span,
      DetectedLambda => unreachable!(),
    }
  }

  /// Checks if the error allows REPL continuation (aka. "..." prompt).
  pub fn allows_continuation(&self) -> bool {
    use ParseError::*;
    match self {
      UnexpectedToken { offending, .. } if offending.kind == TokenType::EOF => true,
      ScanError { error, .. } if error.allows_continuation() => true,
      DetectedLambda => unreachable!(),
      _ => false,
    }
  }
}
