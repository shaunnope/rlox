use std::{borrow::Borrow, mem};

use crate::{
  ast::{
    expr::{self, Expr},
    stmt::{self, Stmt},
  },
  // data::{LoxIdent, LoxValue},
  // data::LoxValue,
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

  // fn parse_program(&mut self) -> Vec<Stmt> {
  //   let mut stmts = Vec::new();
  //   while !self.is_at_end() {
  //     stmts.push(self.parse_decl());
  //   }
  //   stmts
  // }

  fn parse_program(&mut self) -> Vec<Stmt> {
    let mut stmts = Vec::new();
    while !self.is_at_end() {
      if let Ok(expr) = self.parse_expr() {
        stmts.push(Stmt::from(stmt::Expr {
          span: expr.span(),
          expr,
        }))
      }
      // stmts.push(self.parse_decl());
    }
    stmts
  }

  //
  // Statements
  //

  //
  // Expressions
  //

  fn parse_expr(&mut self) -> PResult<Expr> {
    match self.parse_equality() {
      Ok(expr) => Ok(expr),
      Err(error) => {
        println!("{:?}", error);
        self.diagnostics.push(error.clone());
        Err(error)
      }
    }
  }

  // fn parse_sequence(&mut self) -> PResult<Expr> {
  //   let mut expr = self.parse_equality();
  //   loop {
  //     if self.take(TokenType::Comma) {}
  //   }
  // }

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
    self.parse_primary()
  }

  fn parse_primary(&mut self) -> PResult<Expr> {
    use TokenType::*;
    match &self.current_token.kind {
      String(_) | Number(_) | True | False | Nil => {
        let token = self.advance();
        Ok(Expr::from(expr::Lit::from(token.clone())))
      },
      LeftParen => {
        let (expr, span) = self.paired_spanned(
          LeftParen,
          S_MUST,
          "Expected group to be closed",
          |this| this.parse_expr(),
        )?;
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
    self.paired_spanned(
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
