use std::fmt::{self, Display};

use crate::common::{
  error::{Error, ErrorLevel, ErrorType},
  Span
};

// use crate::{data::LoxIdent, span::Span};

#[derive(Debug, Clone)]
pub enum RuntimeError {
  UnsupportedType { message: String, span: Span, level: ErrorLevel },

  // UndefinedVariable { ident: LoxIdent },
  // UnsetVariable { ident: LoxIdent },
  // UndefinedProperty { ident: LoxIdent },
  ZeroDivision { span: Span },
  EmptyStack { span: Span }
}

impl Display for RuntimeError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    use RuntimeError::*;
    match self {
      UnsupportedType { message, span, .. } => {
        write!(f, "{}; at position {}", message, span)
      }

      // UndefinedVariable { ident } => {
      //   write!(
      //     f,
      //     "Undefined variable `{}`; at position {}",
      //     ident.name, ident.span
      //   )
      // }

      // UndefinedProperty { ident } => {
      //   write!(
      //     f,
      //     "Undefined property `{}`; at position {}",
      //     ident.name, ident.span
      //   )
      // }

      // UnsetVariable { ident } => {
      //   write!(
      //     f,
      //     "Variable `{}` uninitialized before access; at position {}",
      //     ident.name, ident.span
      //   )
      // }

      ZeroDivision { span } => {
        write!(f, "Can not divide by zero; at position {}", span)
      },

      EmptyStack { span } => {
        write!(f, "Cannot pop from an empty stack; at position {}", span)
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
      | ZeroDivision { span } 
      | EmptyStack { span }
      => *span,
      // UndefinedVariable { ident } | UnsetVariable { ident } |
      // UndefinedProperty { ident }=> ident.span,
    }
  }
}

impl Error for RuntimeError {
  fn get_level(&self) -> ErrorLevel {
    use RuntimeError::*;
    match self {
      UnsupportedType {level, ..} => level.clone(),
      ZeroDivision {..}
      | EmptyStack {..}
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
