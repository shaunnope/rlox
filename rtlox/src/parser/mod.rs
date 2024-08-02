use std::{borrow::Borrow, mem};

use crate::{
  ast::{
    expr::{self, Expr},
    stmt::{self, Stmt},
  },
  data::{LoxIdent, LoxValue},
  parser::{error::ParseError, scanner::Scanner, state::ParserOptions},
  span::Span,
  token::{Token, TokenType},
};

pub mod error;
pub mod scanner;
pub mod state;

/// Parse result
type PResult<T> = Result<T, ParseError>;

pub type ParserOutcome = (Vec<Stmt>, Vec<ParseError>);

pub struct Parser<'src> {
  scanner: Scanner<'src>,
  current_token: Token,
  prev_token: Token,
  diagnostics: Vec<ParseError>,
  pub options: ParserOptions,
}

impl Parser<'_> {
  pub fn parse(mut self) -> ParserOutcome {
    (self.parse_program(), self.diagnostics)
  }

  fn parse_program(&mut self) -> Vec<Stmt> {
    let mut stmts = Vec::new();
    while !self.is_at_end() {
      stmts.push(self.parse_decl());
    }
    stmts
  }

  //
  // Declarations
  //

  fn parse_decl(&mut self) -> Stmt {
    use TokenType::*;
    let res = match self.current_token.kind {
      Var => self.parse_var_decl(),
      Fun => self.parse_fun_decl(),
      Class => self.parse_class_decl(),
      _ => self.parse_stmt(),
    };

    match res {
      Ok(stmt) => stmt,
      Err(err) => {
        self.diagnostics.push(err);
        self.sync();
        let lo = self.current_token.span.0;
        Stmt::from(stmt::Dummy {
          span: Span::new(lo, lo),
        })
      }
    }
  }

  fn parse_var_decl(&mut self) -> PResult<Stmt> {
    use TokenType::*;
    let var_span = self.consume(Var, S_MUST)?.span;

    let name = self.consume_ident("")?;
    let init = self.take(Equal).then(|| self.parse_expr()).transpose()?;

    let semicolon_span = self
      .consume(Semicolon, "Expected `;` after variable declaration")?
      .span;

    Ok(Stmt::from(stmt::VarDecl {
      span: var_span.to(semicolon_span),
      name,
      init,
    }))
  }

  fn parse_fun_decl(&mut self) -> PResult<Stmt> {
    use TokenType::*;
    let fun_span = self.consume(Fun, S_MUST)?.span;

    let fun = self.parse_fun_params("function", Some(fun_span))?;
    if fun.name.name.starts_with("<lambda") {
      return self.parse_lambda_decl(fun);
    }
    Ok(Stmt::from(fun))
  }

  fn parse_class_decl(&mut self) -> PResult<Stmt> {
    use TokenType::*;
    let class_span = self.consume(Class, S_MUST)?.span;

    let name = self.consume_ident("Expected class name")?;

    let (methods, class_body_span) = self.paired_spanned(
      LeftBrace,
      "Expected `{` before class body", 
      "Expected `}` after class body", 
      |this| {
        let mut methods = Vec::new();
        while !this.is(RightBrace) && !this.is_at_end() {
          methods.push(this.parse_fun_params("method", None)?);
        }

        Ok(methods)
      }
    )?;

    Ok(Stmt::from(stmt::ClassDecl {
      span: class_span.to(class_body_span),
      name,
      // super_name,
      methods,
    }))

  }
  
  fn parse_lambda_decl(&mut self, fun: stmt::FunDecl) -> PResult<Stmt> {
    use TokenType::*;
    let start = fun.span;
    let mut expr = Expr::from(expr::Lambda {
      span: start,
      decl: fun,
    });

    if self.is(LeftParen) {
      loop {
        expr = match self.current_token.kind {
          LeftParen => self.finish_call(expr)?,
          _ => break,
        }
      }
    };

    let semicolon_span = self
      .consume(Semicolon, "Expected `;` after lambda expression")?
      .span;

    let span = start.to(semicolon_span);
    return Ok(Stmt::from(stmt::Expr { span, expr }));
  }

  fn parse_fun_params(
    &mut self,
    kind: &'static str,
    start: Option<Span>,
  ) -> PResult<stmt::FunDecl> {
    use TokenType::*;
    let name = match (
      kind,
      start,
      self.consume_ident(format!("Expected {kind} name")),
    ) {
      (_, _, Ok(ident)) => ident,
      ("function", Some(span), _) => LoxIdent::new_lambda(span),
      ("function", None, _) => unreachable!("Functions should have an associated span"),
      (_, _, Err(err)) => Err(err)?,
    };

    let (params, param_span) = self.paired_spanned(
      TokenType::LeftParen,
      format!("Expected '(' after {} name", kind),
      format!("Expected ')' after {} parameters", kind),
      |this| {
        let mut params = Vec::new();
        if !this.is(RightParen) {
          loop {
            let param = this.consume_ident("Expected parameter name")?;
            params.push(param);
            if !this.take(Comma) {
              break;
            }
          }
        }

        Ok(params)
      },
    )?;

    if params.len() >= 255 {
      self.diagnostics.push(ParseError::Error {
        message: "Can't have more than 255 parameters".into(),
        span: param_span,
      })
    }

    let (body, body_span) = self.parse_block()?;

    Ok(stmt::FunDecl {
      span: start.unwrap_or(name.span).to(body_span),
      name,
      params,
      body,
    })
  }

  //
  // Statements
  //

  fn parse_stmt(&mut self) -> PResult<Stmt> {
    use TokenType::*;
    match self.current_token.kind {
      If => self.parse_if_stmt(),
      While => self.parse_while_stmt(),
      For => self.parse_for_stmt(),
      Print => self.parse_print_stmt(),
      Return => self.parse_return_stmt(),
      LeftBrace => {
        let (stmts, span) = self.parse_block()?;
        Ok(Stmt::from(stmt::Block { span, stmts }))
      }
      _ => self.parse_expr_stmt(),
    }
  }

  fn parse_if_stmt(&mut self) -> PResult<Stmt> {
    let if_span = self.consume(TokenType::If, S_MUST)?.span;
    let (cond, _span) = self.paired_spanned(
      TokenType::LeftParen,
      "Expected '(' after 'if'.",
      "Expected ')' after if condition.",
      |this| this.parse_expr(),
    )?;

    let then_branch = self.parse_stmt()?;
    let else_branch = match self.take(TokenType::Else) {
      true => Some(Box::new(self.parse_stmt()?)),
      false => None,
    };

    Ok(Stmt::from(stmt::If {
      span: if_span.to(match &else_branch {
        Some(br) => br.span(),
        None => then_branch.span(),
      }),
      cond,
      then_branch: then_branch.into(),
      else_branch,
    }))
  }

  fn parse_while_stmt(&mut self) -> PResult<Stmt> {
    let while_span = self.consume(TokenType::While, S_MUST)?.span;
    let (cond, _span) = self.paired_spanned(
      TokenType::LeftParen,
      "Expected '(' after 'if'.",
      "Expected ')' after if condition.",
      |this| this.parse_expr(),
    )?;

    let body = self.parse_stmt()?;
    Ok(Stmt::from(stmt::While {
      span: while_span.to(body.span()),
      cond,
      body: body.into(),
    }))
  }

  /// Desugars `for` loop syntax into other known statements
  fn parse_for_stmt(&mut self) -> PResult<Stmt> {
    use TokenType::*;
    let for_span = self.consume(For, S_MUST)?.span;

    let (init, cond, incr) = self.paired(
      LeftParen,
      "Expected `(` after `for`",
      "Expected `)` to close `for` group",
      |this| {
        let init = match this.current_token.kind {
          Semicolon => {
            this.advance();
            None
          }
          Var => Some(this.parse_var_decl()?),
          _ => Some(this.parse_expr_stmt()?),
        };

        let cond = match this.current_token.kind {
          Semicolon => {
            // No condition => while true
            let lo = this.current_token.span.0;
            Expr::from(expr::Lit {
              span: Span::new(lo, lo),
              value: LoxValue::Boolean(true),
            })
          }
          _ => this.parse_expr()?,
        };
        this.consume(Semicolon, "Expected `;` after `for` condition")?;

        let incr = match this.current_token.kind {
          RightParen => None,
          _ => Some(this.parse_expr()?),
        };

        Ok((init, cond, incr))
      },
    )?;

    let mut body = self.parse_stmt()?;

    // Desugar increment
    if let Some(incr) = incr {
      body = Stmt::from(stmt::Block {
        span: body.span(),
        stmts: vec![
          body,
          Stmt::from(stmt::Expr {
            span: incr.span(),
            expr: incr,
          }),
        ],
      })
    }

    // while
    body = Stmt::from(stmt::While {
      span: for_span.to(body.span()),
      cond,
      body: body.into(),
    });

    // initializer
    if let Some(init) = init {
      body = Stmt::from(stmt::Block {
        span: body.span(),
        stmts: vec![init, body],
      })
    }

    Ok(body)
  }

  fn parse_print_stmt(&mut self) -> PResult<Stmt> {
    let print_token_span = self.consume(TokenType::Print, S_MUST)?.span;
    let expr = self.parse_expr()?;
    let semicolon_span = self
      .consume(TokenType::Semicolon, "Expected `;` after value.")?
      .span;

    Ok(Stmt::from(stmt::Print {
      span: print_token_span.to(semicolon_span),
      expr,
      debug: false,
    }))
  }

  fn parse_return_stmt(&mut self) -> PResult<Stmt> {
    use TokenType::*;
    let return_span = self.consume(Return, S_MUST)?.span;

    let value = (!self.is(Semicolon))
      .then(|| self.parse_expr())
      .transpose()?;

    let semicolon_span = self.consume(Semicolon, "Expected `;` after return")?.span;

    Ok(Stmt::from(stmt::Return {
      span: return_span.to(semicolon_span),
      return_span,
      value,
    }))
  }

  fn parse_block(&mut self) -> PResult<(Vec<Stmt>, Span)> {
    self.paired_spanned(
      TokenType::LeftBrace,
      "Expected block to be opened",
      "Expected block to be closed",
      |this| {
        let mut stmts = Vec::new();
        while !this.is(TokenType::RightBrace) && !this.is_at_end() {
          stmts.push(this.parse_decl());
        }
        Ok(stmts)
      },
    )
  }

  fn parse_expr_stmt(&mut self) -> PResult<Stmt> {
    let expr = self.parse_expr()?;

    // QOL: In repl mode, expressions that do not end with a
    // `;` are evaluated and printed
    if self.options.repl_mode && self.is_at_end() {
      return Ok(Stmt::from(stmt::Print {
        span: expr.span(),
        expr,
        debug: true,
      }));
    }

    let semicolon_span = self
      .consume(TokenType::Semicolon, "Expected `;` after expression.")?
      .span;
    Ok(Stmt::from(stmt::Expr {
      span: expr.span().to(semicolon_span),
      expr,
    }))
  }

  //
  // Expressions
  //

  fn parse_expr(&mut self) -> PResult<Expr> {
    self.parse_sequence()
  }

  fn parse_sequence(&mut self) -> PResult<Expr> {
    let mut expr = self.parse_assignment()?;
    loop {
      if self.take(TokenType::Comma) {
        let operator = self.prev_token.clone();
        let right = self.parse_expr()?;
        expr = Expr::from(expr::Binary {
          span: operator.span,
          left: expr.into(),
          operator,
          right: right.into(),
        })
      } else {
        break Ok(expr);
      }
    }
  }

  fn parse_assignment(&mut self) -> PResult<Expr> {
    let left = self.parse_or()?;

    // expression above is an l-value
    if self.take(TokenType::Equal) {
      let value = self.parse_assignment()?;
      let span = left.span().to(value.span());

      return match left {
        Expr::Var(expr::Var { name, .. }) => {
          Ok(Expr::from(expr::Assignment {
            span,
            name,
            value: value.into(),
          }))
        },
        Expr::Get(expr::Get { name, obj, ..}) => {
          Ok(Expr::from(expr::Set {
            span,
            obj,
            name,
            value: value.into()
          }))
        },
        _ => {
          Err(ParseError::Error {
            message: "Invalid assignment target.".into(),
            span: left.span(),
          })
        }
      }
    }

    Ok(left)
  }

  fn parse_or(&mut self) -> PResult<Expr> {
    bin_expr!(
      self,
      parse_as = Logical,
      token_kinds = Or,
      next_production = parse_and
    )
  }

  fn parse_and(&mut self) -> PResult<Expr> {
    bin_expr!(
      self,
      parse_as = Logical,
      token_kinds = And,
      next_production = parse_equality
    )
  }

  fn parse_equality(&mut self) -> PResult<Expr> {
    bin_expr!(
      self,
      parse_as = Binary,
      token_kinds = EqualEqual | BangEqual,
      next_production = parse_comparison
    )
  }

  fn parse_comparison(&mut self) -> PResult<Expr> {
    bin_expr!(
      self,
      parse_as = Binary,
      token_kinds = Greater | GreaterEqual | Less | LessEqual,
      next_production = parse_term
    )
  }

  fn parse_term(&mut self) -> PResult<Expr> {
    bin_expr!(
      self,
      parse_as = Binary,
      token_kinds = Plus | Minus,
      next_production = parse_factor
    )
  }

  fn parse_factor(&mut self) -> PResult<Expr> {
    bin_expr!(
      self,
      parse_as = Binary,
      token_kinds = Star | Slash,
      next_production = parse_unary
    )
  }

  fn parse_unary(&mut self) -> PResult<Expr> {
    use TokenType::*;
    if let Bang | Minus = self.current_token.kind {
      let operator = self.advance().clone();
      let operand = self.parse_unary()?;
      return Ok(Expr::from(expr::Unary {
        span: operator.span.to(operand.span()),
        operator,
        operand: operand.into(),
      }));
    }
    self.parse_call()
  }

  fn parse_call(&mut self) -> PResult<Expr> {
    use TokenType::*;
    let mut expr = self.parse_lambda()?;
    loop {
      expr = match self.current_token.kind {
        LeftParen => self.finish_call(expr)?,
        Dot => {
          if let Expr::Lambda(_) = expr {
            return Err(ParseError::UnexpectedToken { 
              message: "Unexpected property access on lambda function".into(), 
              offending: self.current_token.clone(), 
              expected: None
            })
          };
          self.advance(); // Consume the `.`
          let name = self.consume_ident("Expected property name after `.`")?;
          Expr::from(expr::Get {
            span: expr.span().to(name.span),
            obj: expr.into(),
            name
          })
        },
        _ => break,
      }
    }

    Ok(expr)
  }

  fn finish_call(&mut self, callee: Expr) -> PResult<Expr> {
    use TokenType::*;
    let (args, call_span) =
      self.paired_spanned(LeftParen, S_MUST, "Expected `)` after arguments", |this| {
        let mut args = Vec::new();
        if !this.is(RightParen) {
          loop {
            args.push(this.parse_assignment()?);
            if !this.take(Comma) {
              break;
            }
          }
        }
        Ok(args)
      })?;

    if args.len() >= 255 {
      // Error isn't thrown because parser is not in a confused state
      self.diagnostics.push(ParseError::Error {
        message: "Call can't have more than 255 arguments".into(),
        span: call_span,
      })
    }

    Ok(Expr::from(expr::Call {
      span: callee.span().to(call_span),
      callee: callee.into(),
      args,
    }))
  }

  fn parse_lambda(&mut self) -> PResult<Expr> {
    use TokenType::*;
    if self.is(TokenType::Fun) {
      let fun_span = self.consume(Fun, S_MUST)?.span;
      let fun = self.parse_fun_params("function", Some(fun_span))?;
      if !fun.name.name.starts_with("<lambda") {
        return Err(ParseError::Error {
          message: format!(
            "Named function {} cannot be used as an expression",
            fun.name
          ),
          span: fun.span,
        });
      }

      return Ok(Expr::from(expr::Lambda {
        span: fun.span,
        decl: fun,
      }));
    }

    self.parse_primary()
  }

  fn parse_primary(&mut self) -> PResult<Expr> {
    use TokenType::*;
    match &self.current_token.kind {
      String(_) | Number(_) | True | False | Nil => {
        let token = self.advance();
        Ok(Expr::from(expr::Lit::from(token.clone())))
      }
      Identifier(_) => {
        let name = self.consume_ident(S_MUST)?;
        Ok(Expr::from(expr::Var {
          span: name.span,
          name,
        }))
      },
      This => {
        let span = self.advance().span;
        Ok(Expr::from(expr::This {
          span,
          name: LoxIdent::new(span, "this")
        }))
      }
      LeftParen => {
        let (expr, span) =
          self.paired_spanned(LeftParen, S_MUST, "Expected group to be closed", |this| {
            this.parse_expr()
          })?;
        Ok(Expr::from(expr::Group {
          span,
          expr: expr.into(),
        }))
      }
      _ => Err(self.unexpected("Expected any expression", None)),
    }
  }
}

// The parser helper methods.
impl<'src> Parser<'src> {
  /// Creates a new parser.
  pub fn new(src: &'src str) -> Self {
    let mut parser = Self {
      scanner: Scanner::new(src),
      current_token: Token::dummy(),
      prev_token: Token::dummy(),
      diagnostics: Vec::new(),
      options: ParserOptions::default(),
    };
    parser.advance(); // The first advancement.
    parser
  }

  /// Advances the parser and returns a reference to the `prev_token` field.
  fn advance(&mut self) -> &Token {
    use TokenType::*;
    let next = loop {
      let maybe_next = self.scanner.next().expect("Cannot advance past EOF.");
      match maybe_next.kind {
        // Report and ignore tokens with the `Error` kind:
        Error(error) => {
          self.diagnostics.push(ParseError::ScanError {
            error,
            span: maybe_next.span,
          });
        }
        // Handle other common ignored kinds:
        Comment(_) | Whitespace(_) => continue,
        _ => break maybe_next,
      };
    };
    self.prev_token = mem::replace(&mut self.current_token, next);
    &self.prev_token
  }

  /// Checks if the current token matches the kind of the given one.
  #[inline]
  fn is(&mut self, expected: impl Borrow<TokenType>) -> bool {
    mem::discriminant(&self.current_token.kind) == mem::discriminant(expected.borrow())
  }

  /// Checks if the current token matches the kind of the given one. In such case advances and
  /// returns true. Otherwise returns false.
  fn take(&mut self, expected: TokenType) -> bool {
    let res = self.is(expected);
    if res {
      self.advance();
    }
    res
  }

  /// Checks if the current token matches the kind of the given one. In such case advances and
  /// returns `Ok(_)` with the consumed token. Otherwise returns an expectation error with the
  /// provided message.
  fn consume(&mut self, expected: TokenType, msg: impl Into<String>) -> PResult<&Token> {
    if self.is(&expected) {
      Ok(self.advance())
    } else {
      Err(self.unexpected(msg, Some(expected)))
    }
  }

  /// Checks if the current token is an identifier. In such case advances and returns `Ok(_)` with
  /// the parsed identifier. Otherwise returns an expectation error with the provided message.
  fn consume_ident(&mut self, msg: impl Into<String>) -> PResult<LoxIdent> {
    let expected = TokenType::Identifier("<ident>".into());
    if self.is(&expected) {
      Ok(LoxIdent::from(self.advance().clone()))
    } else {
      Err(self.unexpected(msg, Some(expected)))
    }
  }

  /// Pair invariant.
  fn paired<I, R>(
    &mut self,
    delim_start: TokenType,
    delim_start_expectation: impl Into<String>,
    delim_end_expectation: impl Into<String>,
    inner: I,
  ) -> PResult<R>
  where
    I: FnOnce(&mut Self) -> PResult<R>,
  {
    self
      .paired_spanned(
        delim_start,
        delim_start_expectation,
        delim_end_expectation,
        inner,
      )
      .map(|(ret, _)| ret)
  }

  /// Pair invariant (2), also returning the full span.
  fn paired_spanned<I, R>(
    &mut self,
    delim_start: TokenType,
    delim_start_expectation: impl Into<String>,
    delim_end_expectation: impl Into<String>,
    inner: I,
  ) -> PResult<(R, Span)>
  where
    I: FnOnce(&mut Self) -> PResult<R>,
  {
    let start_span = self
      .consume(delim_start.clone(), delim_start_expectation)?
      .span;
    let ret = inner(self)?;
    let end_span = match self.consume(delim_start.get_pair(), delim_end_expectation) {
      Ok(token) => token.span,
      Err(error) => {
        return Err(error);
      }
    };
    Ok((ret, start_span.to(end_span)))
  }

  /// Returns an `ParseError::UnexpectedToken`.
  #[inline(always)]
  fn unexpected(&self, message: impl Into<String>, expected: Option<TokenType>) -> ParseError {
    ParseError::UnexpectedToken {
      message: message.into(),
      expected,
      offending: self.current_token.clone(),
    }
  }

  /// Synchronizes parser state to the next statement boundary (generally denoted by `;`)
  ///
  /// TODO: Refactor token types into groups
  fn sync(&mut self) {
    use TokenType::*;
    while !self.is_at_end() {
      match &self.current_token.kind {
        Semicolon => {
          self.advance();
          return;
        }
        Class | For | Fun | If | Print | Return | Var | While => {
          return;
        }
        _ => self.advance(),
      };
    }
  }

  /// Checks if the parser has finished.
  #[inline]
  fn is_at_end(&self) -> bool {
    self.current_token.kind == TokenType::EOF
  }
}

/// (String Must) Indicates the parser to emit a parser error (i.e. the parser is bugged) message.
const S_MUST: &str = "Parser bug. Unexpected token";

/// Parses a binary expression.
macro_rules! bin_expr {
  ($self:expr, parse_as = $ast_kind:ident, token_kinds = $( $kind:ident )|+, next_production = $next:ident) => {{
    let mut expr = $self.$next()?;
    while let $( TokenType::$kind )|+ = $self.current_token.kind {
      let operator = $self.advance().clone();
      let right = $self.$next()?;
      expr = Expr::from(expr::$ast_kind {
        span: expr.span().to(right.span()),
        left: expr.into(),
        operator,
        right: right.into(),
      });
    }
    Ok(expr)
  }};
}
use bin_expr;
