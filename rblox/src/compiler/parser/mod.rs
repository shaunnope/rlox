#[cfg(test)]
mod tests;

use std::{borrow::Borrow, mem};

use rules::ParseFn;

use crate::{
  common::{
    Chunk, Ins, Span
  },
  compiler::{
    parser::{
      error::ParseError,
      rules::{ParseRule, Precedence},
      state::ParserOptions
    },
    scanner::{
      Scanner,
      token::{Token, TokenType}
    },
  }
};

use super::emit;

pub mod error;
pub mod state;
pub mod rules;

/// Parse result
type PResult<T> = Result<T, ParseError>;

pub type ParserOutcome = (Vec<Chunk>, Vec<ParseError>);

pub struct Parser<'src> {
  scanner: Scanner<'src>,
  pub current_token: Token,
  pub prev_token: Token,
  panic_mode: bool,
  chunks: Vec<Chunk>,
  diagnostics: Vec<ParseError>,
  pub _options: ParserOptions,
}

impl Parser<'_> {
  pub fn parse(mut self) -> ParserOutcome {
    self.parse_program();
    (self.chunks, self.diagnostics)
  }

  fn parse_program(&mut self) {
    // while !self.is_at_end() {
    //   self.advance();
    // }
    // stmts
    self.expression();
    
  }

  fn expression(&mut self) {
    if let Err(err) = self.parse_expr() {
      self.diagnostics.push(err)
    }

    if !self.is(TokenType::EOF) {
      self.diagnostics.push(ParseError::UnexpectedToken { 
        message: "Expected end of expression".into(), 
        offending: self.current_token.clone(), 
        expected: Some(TokenType::EOF) 
      })
    }
  }

  fn parse_expr(&mut self) -> PResult<()> {
    self.parse_precedence(Precedence::Assignment)?;
    Ok(())
  }

  fn parse_number(&mut self) -> PResult<()> {
    let prev = self.prev_token.clone();

    if let TokenType::Number(n) = prev.kind {
      emit(Ins::from(n), prev.span, self.current_chunk());
    } else {
      return Err(ParseError::UnexpectedToken { 
        message: "Expected a number".into(), 
        offending: prev, 
        expected: Some(TokenType::Number(0.0)) 
      })
    }
    
    Ok(())
  }

  fn parse_literal(&mut self) -> PResult<()> {
    let prev = self.prev_token.clone();
    use TokenType::*;
    let ins = match prev.kind {
      True => Ins::True,
      False => Ins::False,
      Nil => Ins::Nil,
      _ => unreachable!()
    };

    emit(ins, prev.span, self.current_chunk());
    Ok(())
  }

  fn parse_group(&mut self) -> PResult<()> {
    self.parse_expr()?;
    self.consume(TokenType::RightParen, "Expected `)` after expression")?;
    Ok(())
  }

  fn parse_unary(&mut self) -> PResult<()> {
    let op = self.prev_token.clone();
    self.parse_precedence(Precedence::Unary)?;
    
    let ins = match op.kind {
      TokenType::Minus => Ins::Negate,
      TokenType::Bang => Ins::Not,
      _ => unreachable!()
    };

    emit(ins, op.span, self.current_chunk());

    Ok(())
  }

  fn parse_binary(&mut self) -> PResult<()> {
    let op = self.prev_token.clone();

    let rule = ParseRule::from(&op.kind);
    self.parse_precedence(rule.2.update(1))?;

    use TokenType::*;
    match op.kind {
      Plus => emit(Ins::Add, op.span, self.current_chunk()),
      Minus => emit(Ins::Subtract, op.span, self.current_chunk()),
      Star => emit(Ins::Multiply, op.span, self.current_chunk()),
      Slash => emit(Ins::Divide, op.span, self.current_chunk()),

      BangEqual => {
        emit(Ins::Equal, op.span, self.current_chunk());
        emit(Ins::Not, op.span, self.current_chunk());
      }
      EqualEqual => emit(Ins::Equal, op.span, self.current_chunk()),
      Greater => emit(Ins::Greater, op.span, self.current_chunk()),
      GreaterEqual => {
        emit(Ins::Less, op.span, self.current_chunk());
        emit(Ins::Not, op.span, self.current_chunk());
      },
      Less => emit(Ins::Less, op.span, self.current_chunk()),
      LessEqual => {
        emit(Ins::Greater, op.span, self.current_chunk());
        emit(Ins::Not, op.span, self.current_chunk());
      },

      _ => unreachable!()
    };

    Ok(())
  }

  fn parse_precedence(&mut self, prec: Precedence) -> PResult<()> {
    let prev = self.advance().clone();
    let rule = ParseRule::from(&prev.kind);

    // prefix parser
    self.parse_rule(&rule.0, Err(ParseError::UnexpectedToken { 
      message: "Expected expression".into(), offending: prev, expected: None 
    }))?;

    // infix parser
    let mut other = ParseRule::from(&self.current_token.kind);
    while prec <= other.2 {
      let prev = self.advance();
      let infix = ParseRule::from(&prev.kind).1;
      self.parse_rule(&infix, Ok(()))?;

      other = ParseRule::from(&self.current_token.kind);
    }

    Ok(())
  }

  /// Parse based on 
  fn parse_rule(&mut self, rule: &ParseFn, none_return: PResult<()>) -> PResult<()> {
    use ParseFn as F;
    match rule {
      F::Group => self.parse_group(),
      F::Binary => self.parse_binary(),
      F::Unary => self.parse_unary(),
      F::Number => self.parse_number(),
      F::Literal => self.parse_literal(),
      F::None => none_return
    }
  }

}

// The parser helper methods.
impl<'src> Parser<'src> {
  /// Creates a new parser.
  pub fn new(src: &'src str) -> Self {
    let mut chunks = Vec::new();
    chunks.push(Chunk::new("chunk 0"));
    let mut parser = Self {
      scanner: Scanner::new(src),
      current_token: Token::dummy(),
      prev_token: Token::dummy(),
      panic_mode: false,
      chunks,
      diagnostics: Vec::new(),
      _options: ParserOptions::default(),
    };
    parser.advance(); // The first advancement.
    parser
  }

  pub fn current_chunk(&mut self) -> &mut Chunk {
    self.chunks.last_mut().unwrap()
  }

  /// Advances the parser and returns a reference to the `prev_token` field.
  fn advance(&mut self) -> &Token {
    use TokenType::*;
    let next = loop {
      let maybe_next = self.scanner.next().expect("Cannot advance past EOF.");
      match maybe_next.kind {
        // Report and ignore tokens with the `Error` kind:
        Error(error) => {
          if self.panic_mode {
            continue;
          }
          self.panic_mode = true;
          self.diagnostics.push(ParseError::ScanError {
            error,
            span: maybe_next.span,
          });
        }
        // Handle other common ignored kinds
        Comment(_) | BlockComment(_, _) | Whitespace(_) => continue,
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
  // fn consume_ident(&mut self, msg: impl Into<String>) -> PResult<LoxIdent> {
  //   let expected = TokenType::Identifier("<ident>".into());
  //   if self.is(&expected) {
  //     Ok(LoxIdent::from(self.advance().clone()))
  //   } else {
  //     Err(self.unexpected(msg, Some(expected)))
  //   }
  // }

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

  fn rules(&self, token: TokenType) {

  }
}

/// (String Must) Indicates the parser to emit a parser error (i.e. the parser is bugged) message.
const S_MUST: &str = "Parser bug. Unexpected token";

// /// Parses a binary expression.
// macro_rules! bin_expr {
//   ($self:expr, parse_as = $ast_kind:ident, token_kinds = $( $kind:ident )|+, next_production = $next:ident) => {{
//     let mut expr = $self.$next()?;
//     while let $( TokenType::$kind )|+ = $self.current_token.kind {
//       let operator = $self.advance().clone();
//       let right = $self.$next()?;
//       expr = Expr::from(expr::$ast_kind {
//         span: expr.span().to(right.span()),
//         left: expr.into(),
//         operator,
//         right: right.into(),
//       });
//     }
//     Ok(expr)
//   }};
// }
// use bin_expr;
