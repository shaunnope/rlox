#[cfg(test)]
mod tests;

use std::{borrow::Borrow, mem};

use rules::ParseFn;

use crate::{
  common::{
    data::LoxObject, error::{ErrorLevel, LoxError}, Chunk, Ins, Span
  },
  compiler::{
    Compiler,
    emit,
    emit_loop, 
    parser::{
      error::ParseError,
      rules::{ParseRule, Precedence},
      state::ParserOptions
    }, 
    patch_jump, 
    scanner::{
      token::{Token, TokenType}, Scanner
    }
  }
};

pub mod error;
pub mod state;
pub mod rules;

/// Parse result
pub type PResult<T> = Result<T, ParseError>;

pub type ParserOutcome = (Vec<Chunk>, Vec<ParseError>);

pub struct Parser<'src> {
  scanner: Scanner<'src>,
  pub current_token: Token,
  pub prev_token: Token,
  panic_mode: bool,
  chunks: Vec<Chunk>,
  diagnostics: Vec<ParseError>,
  pub _options: ParserOptions,
  compiler: Compiler
}

impl Parser<'_> {
  pub fn parse(mut self) -> ParserOutcome {
    self.parse_program();
    (self.chunks, self.diagnostics)
  }

  fn parse_program(&mut self) {
    while !self.is_at_end() {
      self.declaration();
    }
  }

  fn declaration(&mut self) {
    use TokenType::*;
    let res = match self.current_token.kind {
      Var => self.var_decl(),
      _ => self.statement()
    };
    if let Err(err) = res {
      self.diagnostics.push(err);
    }

    if self.panic_mode {
      self.sync();
    }
  }

  fn var_decl(&mut self) -> PResult<()> {
    use TokenType::*;
    let var_span = self.consume(Var, S_MUST)?.span;
    let (ident, ident_span) = self.consume_ident("Expected variable name")?;

    if let Err(err) = self.compiler.declare_variable(&ident, ident_span) {
      if err.get_level() > ErrorLevel::Warning {
        return Err(err)
      } else {
        err.report()
      }
    };


    match self.current_token.kind {
      Equal => {
        self.advance();
        self.parse_expr()?;
      },
      _ => {
        emit(Ins::Nil, ident_span, self.current_chunk());
      }
    };

    let semicolon = self.consume(Semicolon, "Expected `;` after variable declaration")?.span;

    self.define_var(ident, var_span.to(semicolon));

    Ok(())
  }

  fn define_var(&mut self, var: LoxObject, span: Span) {
    if let LoxObject::Identifier(name) = var {
      if self.compiler.scope_depth > 0 {
        self.compiler.mark_init();
        return
      }
      emit(Ins::DefGlobal(name), span, self.current_chunk());
    } else {
      unreachable!()
    }
  }

  //
  // Statements
  //

  fn statement(&mut self) -> PResult<()> {
    use TokenType::*;
    match &self.current_token.kind {
      LeftBrace => {
        self.compiler.begin_scope();
        let span = self.parse_block()?;
        self.end_scope(span);
        Ok(())
      },
      If => self.parse_if_stmt(),
      While => self.parse_while(),
      For => self.parse_for(),
      Print => self.parse_print(),
      _ => self.expression()
    }
  }

  /// Parse a block scope
  fn parse_block(&mut self) -> PResult<Span> {
    let (_, span) = self.paired_spanned(
      TokenType::LeftBrace, 
      "Expected block to be opened", 
      "Expected block to be closed", 
      |this| {
        while !this.is(TokenType::RightBrace) && !this.is_at_end() {
          this.declaration();
        }
        Ok(())
      },
    )?;
    Ok(span)
  }

  /// Parse an if statement
  fn parse_if_stmt(&mut self) -> PResult<()> {
    use TokenType::*;
    let if_span = self.consume(If, S_MUST)?.span;
    let (_, cond_span) = self.paired_spanned(
      TokenType::LeftParen,
      "Expected `(` after `if`.",
      "Expected `)` after condition.",
      |this| this.parse_expr(),
    )?;

    let then_jmp = emit(Ins::JumpIfFalse(-1), if_span.to(cond_span), self.current_chunk());
    emit(Ins::Pop, cond_span, self.current_chunk());
    
    let then_span = self.spanned(
      |this| this.statement()
    )?;

    let else_jmp = emit(Ins::Jump(-1), self.prev_token.span, self.current_chunk());

    patch_jump(then_jmp, then_span, self.current_chunk())?;
    emit(Ins::Pop, self.prev_token.span, self.current_chunk());

    let else_span = if self.take(Else) {
      self.spanned(
        |this| this.statement()
      )?
    } else {
      self.prev_token.span
    };

    patch_jump(else_jmp, else_span, self.current_chunk())?;

    Ok(())
  }

  /// Parse a while statement
  fn parse_while(&mut self) -> PResult<()> {
    use TokenType::*;
    let loop_start = self.current_chunk().len();
    let while_span = self.consume(While, S_MUST)?.span;

    let (_, cond_span) = self.paired_spanned(
      TokenType::LeftParen,
      "Expected `(` after `while`.",
      "Expected `)` after condition.",
      |this| this.parse_expr(),
    )?;

    let exit_jmp = emit(Ins::JumpIfFalse(-1), while_span.to(cond_span), self.current_chunk());
    emit(Ins::Pop, cond_span, self.current_chunk());
    let span = self.spanned(
      |this| this.statement()
    )?;
    emit_loop(loop_start, span, self.current_chunk())?;

    patch_jump(exit_jmp, span, self.current_chunk())?;
    emit(Ins::Pop, span, self.current_chunk());
    Ok(())
  }

  /// Parse a for statement
  fn parse_for(&mut self) -> PResult<()> {
    self.compiler.begin_scope();
    use TokenType::*;
    let for_span = self.consume(For, S_MUST)?.span;

    let (loop_start, exit_jmp) = self.paired(
      LeftParen,
      "Expected `(` after `for`",
      "Expected `)` to close `for` group",
      |this| {
        // initializer
        match this.current_token.kind {
          Semicolon => {
            this.advance();
          },
          Var => this.var_decl()?,
          _ => this.expression()?
        };

        let mut loop_start = this.current_chunk().len();

        // condition
        let exit_jmp = match this.current_token.kind {
          Semicolon => None,
          _ => {
            let span = this.parse_expr()?;

            let jmp = emit(Ins::JumpIfFalse(-1), span, this.current_chunk());
            emit(Ins::Pop, span, this.current_chunk());
            Some((jmp, span))
          },
        };
        this.consume(Semicolon, "Expected `;` after `for` condition")?;

        // incrementer
        match this.current_token.kind {
          RightParen => {},
          _ => {
            let body_jmp = emit(Ins::Jump(-1), this.current_token.span, this.current_chunk());
            let inc_start = this.current_chunk().len();
            let inc_span = this.parse_expr()?;
            emit(Ins::Pop, inc_span, this.current_chunk());

            emit_loop(loop_start, inc_span, this.current_chunk())?;
            loop_start = inc_start;
            patch_jump(body_jmp, inc_span, this.current_chunk())?;
          },
        };

        Ok((loop_start, exit_jmp))
      },
    )?;

    self.statement()?;
    emit_loop(
      loop_start, 
      for_span.to(self.current_token.span), 
      self.current_chunk()
    )?;
    if let Some((offset, span)) = exit_jmp {
      patch_jump(offset, span, self.current_chunk())?;
      emit(Ins::Pop, span, self.current_chunk());
    }

    self.compiler.end_scope();
    Ok(())
  }

  /// Parse a print statement.
  fn parse_print(&mut self) -> PResult<()> {
    use TokenType::*;
    let print_span = self.consume(Print, S_MUST)?.span;

    self.parse_expr()?;
    let semicolon_span = self.consume(Semicolon,
    "Expected `;` after value")?.span;

    emit(Ins::Print, print_span.to(semicolon_span), self.current_chunk());

    Ok(())
  }

  /// Parse and consume an expression statement
  fn expression(&mut self) -> PResult<()> {
    let start = self.parse_expr()?;

    let semicolon = self.consume(TokenType::Semicolon, "Expected end of expression")?.span;

    emit(Ins::Pop, start.to(semicolon), self.current_chunk());
    Ok(())
  }

  /// Parse an expression
  fn parse_expr(&mut self) -> PResult<Span> {
    self.parse_precedence(Precedence::Assignment)
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

  fn parse_string(&mut self) -> PResult<()> {
    let prev = self.prev_token.clone();
    match prev.kind {
      TokenType::String(s) => emit(
        Ins::from(LoxObject::String(s)), 
        prev.span, 
        self.current_chunk()
      ),
      _ => unreachable!()
    };
    Ok(())
  }

  fn parse_variable(&mut self, can_assign: bool) -> PResult<()> {
    match &self.prev_token.kind {
      TokenType::Identifier(name) => {
        self.named_variable(
          name.to_owned(), 
          self.prev_token.span,
          can_assign
        )?
      },

      _ => return Err(ParseError::UnexpectedToken { 
        message: "Expected identifier".into(), 
        offending: self.prev_token.clone(), 
        expected: Some(TokenType::Identifier("<ident>".into()))
      })
    };
    Ok(())
  }

  fn named_variable(&mut self, name: impl Into<String>, span: Span, can_assign: bool) -> PResult<()> {
    let name = name.into();
    let arg = self.compiler.resolve_local(&name)?;

    let ins = if can_assign && self.take(TokenType::Equal) {
      self.parse_expr()?;
      match arg {
        Some(n) => Ins::SetLocal(n),
        None => Ins::SetGlobal(name)
      }
    } else {
      match arg {
        Some(n) => Ins::GetLocal(n),
        None => Ins::GetGlobal(name)
      }
    };
    
    emit(ins, span, self.current_chunk());
    Ok(())
  }

  fn parse_and(&mut self) -> PResult<()> {
    let span = self.prev_token.span;
    let end_jmp = emit(Ins::JumpIfFalse(-1), span, self.current_chunk());
    emit(Ins::Pop, span, self.current_chunk());

    let end_span = self.spanned(
      |this| this.parse_precedence(Precedence::And)
    )?;
    patch_jump(end_jmp, end_span, self.current_chunk())?;
    
    Ok(())
  }

  fn parse_or(&mut self) -> PResult<()> {
    let span = self.prev_token.span;
    let else_jmp = emit(Ins::JumpIfFalse(-1), span, self.current_chunk());
    let end_jmp = emit(Ins::Jump(-1), span, self.current_chunk());
    patch_jump(else_jmp, span, self.current_chunk())?;
    emit(Ins::Pop, span, self.current_chunk());

    let end_span = self.spanned(
      |this| this.parse_precedence(Precedence::Or)
    )?;
    patch_jump(end_jmp, end_span, self.current_chunk());

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
        emit(Ins::Not, op.span, self.current_chunk())
      }
      EqualEqual => emit(Ins::Equal, op.span, self.current_chunk()),
      Greater => emit(Ins::Greater, op.span, self.current_chunk()),
      GreaterEqual => {
        emit(Ins::Less, op.span, self.current_chunk());
        emit(Ins::Not, op.span, self.current_chunk())
      },
      Less => emit(Ins::Less, op.span, self.current_chunk()),
      LessEqual => {
        emit(Ins::Greater, op.span, self.current_chunk());
        emit(Ins::Not, op.span, self.current_chunk())
      },

      _ => unreachable!()
    };

    Ok(())
  }

  fn parse_precedence(&mut self, prec: Precedence) -> PResult<Span> {
    let prev = self.advance().clone();
    let rule = ParseRule::from(&prev.kind);
    let start = prev.span;

    // prefix parser
    let can_assign = prec <= Precedence::Assignment;
    self.parse_rule(
      &rule.0, 
      can_assign,
      Err(ParseError::UnexpectedToken { 
      message: "Expected expression".into(), offending: prev, expected: None 
    }))?;

    // infix parser
    let mut other = ParseRule::from(&self.current_token.kind);
    while prec <= other.2 {
      let prev = self.advance();
      let infix = ParseRule::from(&prev.kind).1;
      self.parse_rule(&infix, can_assign, Ok(()))?;

      other = ParseRule::from(&self.current_token.kind);
    }

    if can_assign && self.is(TokenType::Equal) {
      return Err(ParseError::Error { 
        message: "Invalid assignment target".into(), 
        span: self.current_token.span, 
        level: ErrorLevel::Error
      })
    };

    Ok(start.to(self.current_token.span))
  }

  /// Parse based on 
  fn parse_rule(&mut self, rule: &ParseFn, can_assign: bool, none_return: PResult<()>) -> PResult<()> {
    use ParseFn as F;
    match rule {
      F::Group => self.parse_group(),
      F::Binary => self.parse_binary(),
      F::Unary => self.parse_unary(),
      F::Number => self.parse_number(),
      F::Literal => self.parse_literal(),
      F::String => self.parse_string(),
      F::Variable => self.parse_variable(can_assign),
      F::And => self.parse_and(),
      F::Or => self.parse_or(),
      F::None => none_return
    }
  }

}

// The parser helper methods.
impl<'src> Parser<'src> {
  /// Creates a new parser.
  pub fn new(src: &'src str, compiler: Compiler) -> Self {
    let mut chunks = Vec::new();
    chunks.push(Chunk::new("main"));
    let mut parser = Self {
      scanner: Scanner::new(src),
      current_token: Token::dummy(),
      prev_token: Token::dummy(),
      panic_mode: false,
      chunks,
      diagnostics: Vec::new(),
      _options: ParserOptions::default(),
      compiler
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
  fn consume_ident(&mut self, msg: impl Into<String>) -> PResult<(LoxObject, Span)> {
    let expected = TokenType::Identifier("<ident>".into());
    if self.is(&expected) {
      let token = self.advance().clone();
      let span = token.span;
      let obj = LoxObject::try_from(token)?;
      Ok((obj, span))
    } else {
      Err(self.unexpected(msg, Some(expected)))
    }
  }

  /// Get span of parsed section
  fn spanned<I, R>(
    &mut self,
    inner: I
  ) -> PResult<Span> 
  where I: FnOnce(&mut Self) -> PResult<R>,
  {
    let start = self.current_token.span;
    inner(self)?;
    Ok(start.to(self.prev_token.span))
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
    self.panic_mode = false;
  }

  /// Checks if the parser has finished.
  #[inline]
  fn is_at_end(&self) -> bool {
    self.current_token.kind == TokenType::EOF
  }

  fn end_scope(&mut self, span: Span) {
    let count = self.compiler.end_scope();
    emit(Ins::PopN(count), span, self.current_chunk());
  }

}

/// (String Must) Indicates the parser to emit a parser error (i.e. the parser is bugged) message.
const S_MUST: &str = "Parser bug. Unexpected token";
