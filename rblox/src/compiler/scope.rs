
use crate::common::Span;

pub struct Local {
  pub name : String,
  pub span: Span,
  pub depth: i32
}