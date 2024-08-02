use std::fmt::Display;

use crate::{ast::expr, data::LoxIdent, disp::{display_option, display_vec}, span::Span};

make_ast_enum!(
  Stmt,
  [VarDecl, FunDecl, ClassDecl, If, While, Print, Return, Block, Expr, Dummy]
);

#[derive(Debug, Clone)]
pub struct VarDecl {
  pub span: Span,
  pub name: LoxIdent,
  pub init: Option<expr::Expr>,
}

#[derive(Debug, Clone)]
pub struct FunDecl {
  pub span: Span,
  pub name: LoxIdent,
  pub params: Vec<LoxIdent>,
  pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ClassDecl {
  pub span: Span,
  pub name: LoxIdent,
  pub super_name: Option<LoxIdent>,
  pub methods: Vec<FunDecl>,
}

#[derive(Debug, Clone)]
pub struct Return {
  pub span: Span,
  pub return_span: Span,
  pub value: Option<expr::Expr>,
}

#[derive(Debug, Clone)]
pub struct If {
  pub span: Span,
  pub cond: expr::Expr,
  pub then_branch: Box<Stmt>,
  pub else_branch: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct While {
  pub span: Span,
  pub cond: expr::Expr,
  pub body: Box<Stmt>,
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

impl Display for Stmt {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Stmt::*;
    match self {
      Block(block) => write!(f, "Block ( {} )", display_vec(&block.stmts)),
      ClassDecl(class) => write!(f, "Class ( {} {{ \n {:?}\n }}", class.name, class.methods),
      FunDecl(fun) => write!(f, "Fun( {} <{}>  {{ \n {}\n }} )", fun.name, display_vec(&fun.params), display_vec(&fun.body)),
      Return(ret) => write!(f, "Return( {} )", display_option(&ret.value)),

      If(if_stmt) => write!(f, "If( {} ? {} : {} )", if_stmt.cond, if_stmt.then_branch, display_option(&if_stmt.else_branch)),
      Print(print) => write!(f, "Print( {} )", print.expr),
      other => write!(f, "{:#?}", other)
    }
  }
}