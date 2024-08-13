use std::{
  error::Error,
  fmt::{self, Display},
};

use crate::{
  common::{
    error::{LoxError, ErrorLevel, ErrorType}, 
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

  InvalidJump { 
    message: String,
    span: Span 
  },

  StackOverflow { 
    message: String,
    span: Span 
  },

  DetectedLambda,
}

impl Display for ParseError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use ParseError::*;
    match self {
      Error { message, span , ..} |
      StackOverflow { message, span } => {
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

      InvalidJump { message, span } => write!(f, "illegal jump - {message}; at position {span}"),

      DetectedLambda => unreachable!(),
    }
  }
}

impl Error for ParseError {}

impl LoxError for ParseError {
  fn get_level(&self) -> ErrorLevel {
    match self {
      Self::Error { level, ..} => level.clone(),
      _ => ErrorLevel::Error
    }
  }

  fn get_type(&self) -> ErrorType {
    ErrorType::CompileError
  }

  fn get_span(&self) -> Span {
    self.primary_span()
  }
}

impl ParseError {
  /// Returns the span that caused the error.
  pub fn primary_span(&self) -> Span {
    use ParseError::*;
    match self {
      Error { span, .. } | 
      ScanError { span, .. } | 
      InvalidJump { span, ..} |
      StackOverflow { span, .. }
      => *span,
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
