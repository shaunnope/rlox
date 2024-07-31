use crate::{ast::expr, data::LoxIdent, span::Span};

make_ast_enum!(Stmt, [VarDecl, Print, Block, Expr, Dummy]);

#[derive(Debug, Clone)]
pub struct VarDecl {
  pub span: Span,
  pub name: LoxIdent,
  pub init: Option<expr::Expr>,
}

#[derive(Debug, Clone)]
pub struct Print {
  pub span: Span,
  pub expr: expr::Expr,
  pub debug: bool,
}

#[derive(Debug, Clone)]
pub struct Block {
  pub span: Span,
  pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Expr {
  pub span: Span,
  pub expr: expr::Expr,
}

/// For error purposes.
#[derive(Debug, Clone)]
pub struct Dummy {
  pub span: Span,
}
