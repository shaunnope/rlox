use std::{error::Error, fmt::{self, Display}};

use crate::common::{
  error::{LoxError, ErrorLevel, ErrorType},
  Span
};

// use crate::{data::LoxIdent, span::Span};

#[derive(Debug, Clone)]
pub enum RuntimeError {
  UnsupportedType { message: String, span: Span, level: ErrorLevel },

  UndefinedVariable { name: String, span: Span },
  // UndefinedProperty { ident: LoxIdent },
  ZeroDivision(Span),
  EmptyStack(Span),
  StackOverflow(Span) // TODO: distinguish between call stack and vm stack
}

impl Display for RuntimeError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use RuntimeError::*;
    match self {
      UnsupportedType { message, span, .. } => {
        write!(f, "{}; at position {}", message, span)
      }

      UndefinedVariable { name, span } => {
        write!(
          f,
          "Undefined variable `{}`; at position {}",
          name, span
        )
      }

      // UndefinedProperty { ident } => {
      //   write!(
      //     f,
      //     "Undefined property `{}`; at position {}",
      //     ident.name, ident.span
      //   )
      // }

      ZeroDivision(span) => {
        write!(f, "Can not divide by zero; at position {}", span)
      },

      EmptyStack(span) => {
        write!(f, "Cannot pop from an empty stack; at position {}", span)
      },
      StackOverflow(span) => {
        write!(f, "stack overflow; at position {}", span)
      }
    }
  }
}

impl RuntimeError {
  /// Returns the span that caused the error.
  pub fn primary_span(&self) -> Span {
    use RuntimeError::*;
    match self {
      UnsupportedType { span, .. } 
      | UndefinedVariable { span, ..}
      | ZeroDivision(span) 
      | EmptyStack(span)
      | StackOverflow(span)
      => *span,
      // UndefinedProperty { ident }=> ident.span,
    }
  }
}

impl Error for RuntimeError {}

impl LoxError for RuntimeError {
  fn get_level(&self) -> ErrorLevel {
    use RuntimeError::*;
    match self {
      UnsupportedType {level, ..} => level.clone(),
      ZeroDivision(_)
      | EmptyStack(_)
      | StackOverflow(_)
      | UndefinedVariable {..}
      => ErrorLevel::Error,
    }
  }

  fn get_type(&self) -> ErrorType {
    ErrorType::RuntimeError
  }

  fn get_span(&self) -> Span {
    self.primary_span()
  }
}
