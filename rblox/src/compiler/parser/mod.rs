#[cfg(test)]
mod tests;

use std::{borrow::Borrow, cell::{RefCell, RefMut}, mem, rc::Rc};

use rules::ParseFn;

use crate::{
  common::{
    data::LoxObject, 
    error::{ErrorLevel, LoxError}, 
    Ins, Span
  },
  compiler::{
    parser::{
      error::ParseError,
      rules::{ParseRule, Precedence},
      state::ParserOptions
    }, 
    scanner::{
      token::{Token, TokenType}, Scanner
    }, 
    scope::{Module, Push},
    Compiler, FunctionType
  }
};

pub mod error;
pub mod state;
pub mod rules;

/// Parse result
pub type PResult<T> = Result<T, ParseError>;

pub type ParserOutcome = Vec<ParseError>;

pub struct Parser<'src> {
  scanner: Scanner<'src>,
  pub current_token: Token,
  pub prev_token: Token,
  panic_mode: bool,
  diagnostics: Vec<ParseError>,
  pub _options: ParserOptions,
  compiler: RefCell<Compiler>,
  module: Rc<RefCell<Module>>
}

impl Parser<'_> {
  const MAX_ARGS: usize = 255;
  pub fn parse(mut self) -> ParserOutcome {
    self.parse_program();
    self.emit_return();

    let main = self.compiler.into_inner().function;
    self.module.borrow_mut().push(main);
    self.diagnostics
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
      Fun => self.fun_decl(),
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

    if let Err(err) = self.current().declare_variable(&ident, ident_span) {
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
        self.current().emit(Ins::Nil, ident_span);
      }
    };

    let semicolon = self.consume(Semicolon, "Expected `;` after variable declaration")?.span;

    self.define_var(ident, var_span.to(semicolon));

    Ok(())
  }

  fn define_var(&mut self, var: LoxObject, span: Span) {
    if let LoxObject::Identifier(name) = var {
      if self.current().scope_depth > 0 {
        self.current().mark_init();
        return
      }
      self.current().emit(Ins::DefGlobal(name), span);
    } else {
      unreachable!()
    }
  }

  fn fun_decl(&mut self) -> PResult<()> {
    use TokenType::*;
    let fun_span = self.consume(Fun, S_MUST)?.span;
    let (ident, ident_span) = self.consume_var("Expected function name")?;

    self.current().mark_init();
    self.function(ident.data(), FunctionType::Function, fun_span)?;
    self.define_var(ident, ident_span);


    Ok(())
  }

  /// Parse function params and body
  fn function(&mut self, name: impl Into<String>, kind: FunctionType, span: Span) -> PResult<()> {
    let name = name.into();
    let enclosing = self.compiler.replace(
      Compiler::build(&name, kind)
    );
    // does not have a corresponding `end_scope` because the enclosed compiler
    // ends after the function body is parsed
    self.current().begin_scope();

    self.paired(
      TokenType::LeftParen, 
      "Expected `(` after function name", 
      "Expected `)` after parameters", 
      |this| {
        if this.is(TokenType::RightParen) {
          return Ok(())
        }
        let start = this.prev_token.span;
        loop {
          this.current().function.arity += 1;
          if this.current().function.arity > Self::MAX_ARGS {
            return Err(ParseError::Error { 
              level: ErrorLevel::Error, 
              message: format!("Can't have more than {} parameters", Self::MAX_ARGS), 
              span: start.to(this.current_token.span) 
            })
          }
          let (param, span) = this.consume_var("Expected parameter name")?;
          this.define_var(param, span);

          if !this.take(TokenType::Comma) {
            break;
          }
        }
        Ok(())
      },
    )?;
    let block_span = self.parse_block()?;

    let func = self.compiler.replace(enclosing).function;
    let func = self.module.borrow_mut().push(func);
    self.current().emit(Ins::from(LoxObject::Function(name, func)), span.to(block_span));
    
    Ok(())
  }

  //
  // Statements
  //

  fn statement(&mut self) -> PResult<()> {
    use TokenType::*;
    match &self.current_token.kind {
      LeftBrace => {
        self.current().begin_scope();
        let span = self.parse_block()?;
        self.end_scope(span);
        Ok(())
      },
      If => self.parse_if_stmt(),
      While => self.parse_while(),
      For => self.parse_for(),
      Print => self.parse_print(),
      Return => self.parse_return(),
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

    let then_jmp = self.current().emit(Ins::JumpIfFalse(-1), if_span.to(cond_span));
    self.current().emit(Ins::Pop, cond_span);
    
    let then_span = self.spanned(
      |this| this.statement()
    )?;

    let prev_span = self.prev_token.span;
    let else_jmp = self.current().emit(Ins::Jump(-1), prev_span);

    self.current().patch_jump(then_jmp, then_span)?;
    self.current().emit(Ins::Pop, prev_span);

    let else_span = if self.take(Else) {
      self.spanned(
        |this| this.statement()
      )?
    } else {
      self.prev_token.span
    };

    self.current().patch_jump(else_jmp, else_span)?;

    Ok(())
  }

  /// Parse a while statement
  fn parse_while(&mut self) -> PResult<()> {
    use TokenType::*;
    let loop_start = chunk!(self).len();
    let while_span = self.consume(While, S_MUST)?.span;

    let (_, cond_span) = self.paired_spanned(
      TokenType::LeftParen,
      "Expected `(` after `while`.",
      "Expected `)` after condition.",
      |this| this.parse_expr(),
    )?;

    let exit_jmp = self.current().emit(Ins::JumpIfFalse(-1), while_span.to(cond_span));
    self.current().emit(Ins::Pop, cond_span);
    let span = self.spanned(
      |this| this.statement()
    )?;
    self.current().emit_loop(loop_start, span)?;

    self.current().patch_jump(exit_jmp, span)?;
    self.current().emit(Ins::Pop, span);
    Ok(())
  }

  /// Parse a for statement
  fn parse_for(&mut self) -> PResult<()> {
    self.current().begin_scope();
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

        let mut loop_start = chunk!(this).len();

        // condition
        let exit_jmp = match this.current_token.kind {
          Semicolon => None,
          _ => {
            let span = this.parse_expr()?;

            let jmp = this.current().emit(Ins::JumpIfFalse(-1), span);
            this.current().emit(Ins::Pop, span);
            Some((jmp, span))
          },
        };
        this.consume(Semicolon, "Expected `;` after `for` condition")?;

        // incrementer
        match this.current_token.kind {
          RightParen => {},
          _ => {
            let span = this.current_token.span;
            let body_jmp = this.current().emit(Ins::Jump(-1), span);
            let inc_start = chunk!(this).len();
            let inc_span = this.parse_expr()?;
            this.current().emit(Ins::Pop, inc_span);

            this.current().emit_loop(loop_start, inc_span)?;
            loop_start = inc_start;
            this.current().patch_jump(body_jmp, inc_span)?;
          },
        };

        Ok((loop_start, exit_jmp))
      },
    )?;

    self.statement()?;
    let span = self.current_token.span;
    self.current().emit_loop(
      loop_start, 
      for_span.to(span), 
    )?;
    if let Some((offset, span)) = exit_jmp {
      self.current().patch_jump(offset, span)?;
      self.current().emit(Ins::Pop, span);
    }

    self.current().end_scope();
    Ok(())
  }

  /// Parse a print statement
  fn parse_print(&mut self) -> PResult<()> {
    use TokenType::*;
    let print_span = self.consume(Print, S_MUST)?.span;

    self.parse_expr()?;
    let semicolon_span = self.consume(Semicolon,
    "Expected `;` after value")?.span;

    self.current().emit(Ins::Print, print_span.to(semicolon_span));

    Ok(())
  }

  /// Parse a return statement
  fn parse_return(&mut self) -> PResult<()> {
    use TokenType::*;
    let return_span = self.consume(Return, S_MUST)?.span;
    if self.current().fun_type == FunctionType::Script {
      return Err(ParseError::Error { 
        level: ErrorLevel::Warning, 
        message: "Detected return from top-level code".into(), 
        span: return_span
      })
    }

    if self.take(Semicolon) {
      self.emit_return();
    } else {
      self.parse_expr()?;
      let span = self.consume(Semicolon, "Expected `;` after return value")?.span;
      self.current().emit(Ins::Return, return_span.to(span));
    }

    Ok(())
  }

  /// Parse and consume an expression statement
  fn expression(&mut self) -> PResult<()> {
    let start = self.parse_expr()?;

    let semicolon = self.consume(TokenType::Semicolon, "Expected end of expression")?.span;

    self.current().emit(Ins::Pop, start.to(semicolon));
    Ok(())
  }

  /// Parse an expression
  fn parse_expr(&mut self) -> PResult<Span> {
    self.parse_precedence(Precedence::Sequence)
  }

  fn parse_number(&mut self) -> PResult<()> {
    let prev = self.prev_token.clone();

    if let TokenType::Number(n) = prev.kind {
      self.current().emit(Ins::from(n), prev.span);
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

    self.current().emit(ins, prev.span);
    Ok(())
  }

  fn parse_string(&mut self) -> PResult<()> {
    let prev = self.prev_token.clone();
    match prev.kind {
      TokenType::String(s) => self.current().emit(
        Ins::from(LoxObject::String(s)), 
        prev.span, 
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
    let arg = self.current().resolve_local(&name)?;

    let ins = if can_assign && self.take(TokenType::Equal) {
      self.parse_precedence(Precedence::Assignment)?;
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
    
    self.current().emit(ins, span);
    Ok(())
  }

  fn parse_call(&mut self) -> PResult<()> {
    let open = self.prev_token.span;
    let (args, close) = self.argument_list()?;
    self.current().emit(Ins::Call(args), open.to(close));
    Ok(())
  }

  fn argument_list(&mut self) -> PResult<(usize, Span)> {
    let start = self.prev_token.span;
    let mut count = 0;
    if !self.is(TokenType::RightParen) {
      loop {
        self.parse_precedence(Precedence::Assignment)?;
        if count == Self::MAX_ARGS {
          return Err(ParseError::Error { 
            level: ErrorLevel::Error, 
            message: "Can't have more than 255 arguments".into(), 
            span: start.to(self.prev_token.span) 
          })
        }
        count += 1;
        if !self.take(TokenType::Comma) {
          break;
        }
      }
    }
    let span = self.consume(TokenType::RightParen, "Expected `)` after arguments")?.span;
    Ok((count, span))
  }
  
  fn parse_and(&mut self) -> PResult<()> {
    let span = self.prev_token.span;
    let end_jmp = self.current().emit(Ins::JumpIfFalse(-1), span);
    self.current().emit(Ins::Pop, span);

    let end_span = self.spanned(
      |this| this.parse_precedence(Precedence::And)
    )?;
    self.current().patch_jump(end_jmp, end_span)?;
    
    Ok(())
  }

  fn parse_or(&mut self) -> PResult<()> {
    let span = self.prev_token.span;
    let else_jmp = self.current().emit(Ins::JumpIfFalse(-1), span);
    let end_jmp = self.current().emit(Ins::Jump(-1), span);
    self.current().patch_jump(else_jmp, span)?;
    self.current().emit(Ins::Pop, span);

    let end_span = self.spanned(
      |this| this.parse_precedence(Precedence::Or)
    )?;
    self.current().patch_jump(end_jmp, end_span)?;

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

    self.current().emit(ins, op.span);

    Ok(())
  }

  fn parse_binary(&mut self, can_seq: bool) -> PResult<()> {
    use TokenType::*;
    let op = self.prev_token.clone();

    let rule = ParseRule::from(&op.kind);
    if can_seq && op.kind == Comma {
      return Ok(())
    }
    self.parse_precedence(rule.2.update(1))?;
    
    match op.kind {
      Comma => unreachable!(),
      Plus => self.current().emit(Ins::Add, op.span),
      Minus => self.current().emit(Ins::Subtract, op.span),
      Star => self.current().emit(Ins::Multiply, op.span),
      Slash => self.current().emit(Ins::Divide, op.span),

      BangEqual => {
        self.current().emit(Ins::Equal, op.span);
        self.current().emit(Ins::Not, op.span)
      }
      EqualEqual => self.current().emit(Ins::Equal, op.span),
      Greater => self.current().emit(Ins::Greater, op.span),
      GreaterEqual => {
        self.current().emit(Ins::Less, op.span);
        self.current().emit(Ins::Not, op.span)
      },
      Less => self.current().emit(Ins::Less, op.span),
      LessEqual => {
        self.current().emit(Ins::Greater, op.span);
        self.current().emit(Ins::Not, op.span)
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
    self.parse_rule(
      &rule.0, 
      &prec,
      Err(ParseError::UnexpectedToken { 
      message: "Expected expression".into(), offending: prev, expected: None 
    }))?;

    // infix parser
    let mut other = ParseRule::from(&self.current_token.kind);
    while prec <= other.2 {
      let prev = self.advance();
      let infix = ParseRule::from(&prev.kind).1;
      self.parse_rule(&infix, &prec, Ok(()))?;

      other = ParseRule::from(&self.current_token.kind);
    }

    if prec <= Precedence::Assignment && self.is(TokenType::Equal) {
      return Err(ParseError::Error { 
        message: "Invalid assignment target".into(), 
        span: self.current_token.span, 
        level: ErrorLevel::Error
      })
    };

    if prec <= Precedence::Sequence && self.prev_token.kind == TokenType::Comma {
      let span = self.prev_token.span;
      self.current().emit(Ins::Pop, span);
      self.parse_expr()?;
    }

    Ok(start.to(self.current_token.span))
  }

  /// Parse according to given rule.
  fn parse_rule(&mut self, rule: &ParseFn, prec: &Precedence, none_return: PResult<()>) -> PResult<()> {
    use ParseFn as F;
    match rule {
      F::Group => self.parse_group(),
      F::Binary => self.parse_binary(*prec <= Precedence::Sequence),
      F::Unary => self.parse_unary(),
      F::Number => self.parse_number(),
      F::Literal => self.parse_literal(),
      F::String => self.parse_string(),
      F::Variable => self.parse_variable(*prec <= Precedence::Assignment),
      F::Call => self.parse_call(),
      F::And => self.parse_and(),
      F::Or => self.parse_or(),
      F::None => none_return
    }
  }

}

// The parser helper methods.
impl<'src> Parser<'src> {
  /// Creates a new parser.
  pub fn new(src: &'src str, module: Rc<RefCell<Module>>) -> Self {
    let mut parser = Self {
      scanner: Scanner::new(src),
      current_token: Token::dummy(),
      prev_token: Token::dummy(),
      panic_mode: false,
      diagnostics: Vec::new(),
      _options: ParserOptions::default(),
      compiler: RefCell::new(Compiler::new()),
      module
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

  /// Consumes the next identifier and declares it as a variable
  fn consume_var(&mut self, msg: impl Into<String>) -> PResult<(LoxObject, Span)> {
    let (ident, ident_span) = self.consume_ident(msg)?;

    if let Err(err) = self.current().declare_variable(&ident, ident_span) {
      if err.get_level() > ErrorLevel::Warning {
        return Err(err)
      } else {
        err.report()
      }
    };
    Ok((ident, ident_span))
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

  

}

/// Get a mutable reference to the current chunk
macro_rules! chunk {
  ($self:ident) => {
    &mut $self.current().function.chunk
  };
}

use chunk;

/// Compiler wrappers
impl Parser<'_> {

  #[inline]
  fn current(&mut self) -> RefMut<Compiler> {
    self.compiler.borrow_mut()
  }

  fn end_scope(&mut self, span: Span) {
    let count = self.current().end_scope();
    self.current().emit(Ins::PopN(count), span);
  }

  /// Emit an implicit return `nil` at the end of a function body
  fn emit_return(&mut self) {
    let span = self.prev_token.span;
    self.current().emit(Ins::Nil, span);
    self.current().emit(Ins::Return, span);
  }


}

/// (String Must) Indicates the parser to emit a parser error (i.e. the parser is bugged) message.
const S_MUST: &str = "Parser bug. Unexpected token";
