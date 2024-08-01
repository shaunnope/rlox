
use crate::span::Span;

#[derive(Debug)]
pub enum ErrorType {
  Error,
  Warning
}

#[derive(Debug)]
pub struct ResolveError {
  pub kind: ErrorType,
  pub message: String,
  pub span: Span,
}