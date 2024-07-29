
use crate::token::{Token, TokenType};
use crate::ast;

use crate::error::Error;

struct State {
  tokens: Vec<Token>,
  current: usize
}

impl State {
  /// Construct a new State instance
  fn build(tokens: Vec<Token>) -> Self {
    State {tokens, current: 0}
  }

  fn at_end(&self) -> bool {
    self.peek().ttype == TokenType::EOF
  }

  fn peek(&self) -> &Token {
    &self.tokens[self.current]
  }

  fn previous(&mut self) -> &Token {
    &self.tokens[self.current-1]
  }

  fn advance(&mut self) {
    if !self.at_end() {
      self.current += 1;
    }
  }

  /// Advance state if token match, else error
  fn consume(&mut self, token: TokenType, message: &str)  {
    if self.peek().ttype == token {
      self.advance();
      return
    }
    self.peek().error(message);
  }

  /// Synchronise state to next statement boundary
  /// 
  /// TODO: Refactor token types into groups
  fn _sync(&mut self) {
    self.advance();

    while !self.at_end() {
      if self.previous().ttype == TokenType::Semicolon {return};
      match self.peek().ttype {
        TokenType::Class | TokenType::Fun | TokenType::Var | TokenType::For |
        TokenType::If | TokenType::While | TokenType::Print | TokenType::Return
          => return,
        _ => {}
      }
      self.advance();
    }
  }
}

type Tokens<'a> = &'a mut State;
type MaybeExpr = Result<ast::Expr, Error>;

pub fn parse(tokens: Vec<Token>) -> Option<ast::Expr> {
  let mut state = State::build(tokens);

  expression(&mut state).ok()
}

fn expression(tokens: Tokens) -> MaybeExpr {
  sequence(tokens)
}

fn sequence(state: Tokens) -> MaybeExpr {
  let mut expr = equality(state)?;

  loop {
    let token = state.peek();
    match token.ttype {
      TokenType::Comma => {
        state.advance();
        let op = state.previous().clone();
        let right = Box::new(expression(state)?);
        expr = ast::Expr::Binary { left: Box::new(expr), op, right};
      },
      _ => break
    }
  }

  Ok(expr)
}

fn equality(state: Tokens) -> MaybeExpr {
  let mut expr = comparison(state)?;

  loop {
    let token = state.peek();
    match token.ttype {
      TokenType::BangEqual | TokenType::EqualEqual => {
        state.advance();
        let op = state.previous().clone();
        let right = Box::new(comparison(state)?);
        expr = ast::Expr::Binary { left: Box::new(expr), op, right};
      },
      _ => break
    }
  }

  Ok(expr)
}

fn comparison(state: Tokens) -> MaybeExpr {
  let mut expr = term(state)?;

  loop {
    let token = state.peek();
    match token.ttype {
      TokenType::Greater | TokenType::GreaterEqual |
      TokenType::Less | TokenType::LessEqual => {
        state.advance();
        let op = state.previous().clone();
        let right = Box::new(term(state)?);
        expr = ast::Expr::Binary { left: Box::new(expr), op, right};
      },
      _ => break
    }
  }
  
  Ok(expr)
}

fn term(state: Tokens) -> MaybeExpr {
  let mut expr = factor(state)?;

  loop {
    let token = state.peek();
    match token.ttype {
      TokenType::Minus | TokenType::Plus => {
        state.advance();
        let op = state.previous().clone();
        let right = Box::new(factor(state)?);
        expr = ast::Expr::Binary { left: Box::new(expr), op, right};
      },
      _ => break
    }
  }
  
  Ok(expr)
}

fn factor(state: Tokens) -> MaybeExpr {
  let mut expr = unary(state)?;

  loop {
    let token = state.peek();
    match token.ttype {
      TokenType::Slash | TokenType::Star => {
        state.advance();
        let op = state.previous().clone();
        let right = Box::new(unary(state)?);
        expr = ast::Expr::Binary { left: Box::new(expr), op, right};
      },
      _ => break
    }
  }
  
  Ok(expr)
}

fn unary(state: Tokens) -> MaybeExpr {

  let token = state.peek();
  match token.ttype {
    TokenType::Bang | TokenType::Minus => {
      state.advance();
      let op = state.previous().clone();
      let right = Box::new(unary(state)?);
      
      Ok(ast::Expr::Unary { op, right})
    }
    _ => primary(state)
  }
}

fn primary(state: Tokens) -> MaybeExpr {
  let token = state.peek().ttype.clone();
  match token {
    TokenType::False | TokenType::True | TokenType::Nil |
    TokenType::Number(_) | TokenType::String(_) => {
      state.advance();
      return Ok(ast::Expr::Literal(token))
    },
    TokenType::LeftParen => {
      state.advance();
      let expr = Box::new(expression(state)?);
      state.consume(TokenType::RightParen, "Expect ')' after expression.");
      return Ok(ast::Expr::Grouping(expr))
    }
    _ => Err(state.peek().error("Expect expression."))
  }
}

// fn match_left_associative(state: Tokens, matched: impl Fn(&TokenType) -> bool) {
//   let mut expr = term(state);

//   loop {
//     let token = state.peek();
//     if matched(&token.ttype) {
//       state.advance();
//       let op = state.previous().clone();
//       let right = Box::new(term(state));
//       expr = ast::Expr::Binary { left: Box::new(expr), op, right};
//     }
//     match token.ttype {
//       TokenType::Greater | TokenType::GreaterEqual |
//       TokenType::Less | TokenType::LessEqual => {
        
//       },
//       _ => break
//     }
//   }
// }