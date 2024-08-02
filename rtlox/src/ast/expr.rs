use std::fmt;

use crate::{
  ast::stmt,
  data::{LoxIdent, LoxValue},
  disp::display_vec,
  span::Span,
  token::{Token, TokenType},
};

make_ast_enum!(
  Expr,
  [Assignment, Var, Lambda, Call, Get, Set, This, Super, Lit, Group, Unary, Binary, Logical]
);

#[derive(Debug, Clone)]
pub struct Assignment {
  pub span: Span,
  pub name: LoxIdent,
  pub value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Var {
  pub span: Span,
  pub name: LoxIdent,
}

#[derive(Debug, Clone)]
pub struct Lambda {
  pub span: Span,
  pub decl: stmt::FunDecl,
}

#[derive(Debug, Clone)]
pub struct Call {
  pub span: Span,
  pub callee: Box<Expr>,
  pub args: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct Get {
  pub span: Span,
  pub obj: Box<Expr>, 
  pub name: LoxIdent,
}

#[derive(Debug, Clone)]
pub struct Set {
  pub span: Span,
  pub obj: Box<Expr>, 
  pub name: LoxIdent,
  pub value: Box<Expr>
}

#[derive(Debug, Clone)]
pub struct This {
  pub span: Span,
  pub name: LoxIdent,
}

#[derive(Debug, Clone)]
pub struct Super {
  pub span: Span,
  pub super_ident: LoxIdent,
  pub method: LoxIdent,
}

#[derive(Debug, Clone)]
pub struct Lit {
  pub span: Span,
  pub value: LoxValue,
}

#[derive(Debug, Clone)]
pub struct Group {
  pub span: Span,
  pub expr: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Unary {
  pub span: Span,
  pub operator: Token,
  pub operand: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Binary {
  pub span: Span,
  pub left: Box<Expr>,
  pub operator: Token,
  pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct Logical {
  pub span: Span,
  pub left: Box<Expr>,
  pub operator: Token,
  pub right: Box<Expr>,
}

//
// Some other utilities.
//

impl From<Token> for Lit {
  fn from(token: Token) -> Self {
    use LoxValue as L;
    use TokenType as T;
    Lit {
      span: token.span,
      value: match token.kind {
        T::String(string) => L::String(string),
        T::Number(number) => L::Number(number),
        T::Nil => L::Nil,
        T::True => L::Boolean(true),
        T::False => L::Boolean(false),
        unexpected => unreachable!(
          "Invalid `Token` ({:?}) to `Literal` conversion.",
          unexpected
        ),
      },
    }
  }
}

impl fmt::Display for Expr {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Var(var) => write!(f, "{}", var.name),
      Self::Lit(token) => write!(f, "{}", token.value),
      Self::Group(node) => write!(f, "(group {})", node.expr),
      Self::Binary(bin) => {
        return write!(f, "({} {} {})", bin.operator, bin.left, bin.right)
      },
      Self::Logical(logical) => {
        return write!(f, "({} {} {})", logical.operator, logical.left, logical.right)
      },
      Self::Unary(unary) => {
        return write!(f, "({} {})", unary.operator, unary.operand)
      },
      Self::Assignment(assign) => write!(f, "(= {} {})", assign.name, assign.value),
      Self::Call(call) => write!(f, "(call {} {})", call.callee, display_vec(&call.args)),
      Self::Get(get) => write!(f, "(get {} {:?})", get.name, get.obj),
      Self::Set(set) => write!(f, "(set {} {} {:?})", set.name, set.value, set.obj),
      Self::Lambda(lambda) => write!(f, "(L {} {:?} {:?})", lambda.decl.name, lambda.decl.params, lambda.decl.body),
      Self::This(this) => write!(f, "(this {})", this.name),
      Self::Super(class) => write!(f, "(super {} {})", class.super_ident, class.method),
    }
  }
}