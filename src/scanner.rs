use std::{iter::Peekable, str::Chars};

use crate::token::{Token, TokenType};

pub fn scan_tokens(source: &str) -> Vec<Token> {

  let mut res = Vec::new();
  let tokens = &mut res;

  let mut line = 1;
  // let mut start = 0;

  let iter = &mut source.chars().peekable();

  while let Some(ch) = iter.next() {
    match ch {
      // single char
      '(' => add_token(line, tokens, TokenType::LeftParen),
      ')' => add_token(line, tokens, TokenType::RightParen),
      '{' => add_token(line, tokens, TokenType::LeftBrace),
      '}' => add_token(line, tokens, TokenType::RightBrace),
      ',' => add_token(line, tokens, TokenType::Comma),
      '.' => add_token(line, tokens, TokenType::Dot),
      '-' => add_token(line, tokens, TokenType::Minus),
      '+' => add_token(line, tokens, TokenType::Plus),
      ';' => add_token(line, tokens, TokenType::Semicolon),
      '*' => add_token(line, tokens, TokenType::Star),
      
      // operators
      '!' => add_token(
        line, tokens, 
        if match_next(iter, &'=') {
          iter.next();
          TokenType::BangEqual
        } else {
          TokenType::Bang
        }
      ),
      '=' => add_token(
        line, tokens, 
        if match_next(iter, &'=') {
          iter.next();
          TokenType::EqualEqual
        } else {
          TokenType::Equal
        }
      ),
      '<' => add_token(
        line, tokens, 
        if match_next(iter, &'=') {
          iter.next();
          TokenType::LessEqual
        } else {
          TokenType::Less
        }
      ),
      '>' => add_token(
        line, tokens, 
        if match_next(iter, &'=') {
          iter.next();
          TokenType::GreaterEqual
        } else {
          TokenType::Greater
        }
      ),

      // slash
      '/' => {
        if match_next(iter, &'/') {
          while let Some(ch) = iter.next() {
            if match_next(iter, &'\n') {
              break
            }
          }
        } else {
          add_token(line, tokens, TokenType::Slash)
        }
      }
      
      // whitespace (ignored)
      ' ' | '\r' | '\t' => {},

      // newline
      '\n' => line += 1,
      _ => crate::error(line, "Unexpected character")
    };
    
  }

  tokens.push(
    Token { 
      ttype: TokenType::EOF, 
      line 
    });

  return res
}

// fn scan_token(source: &str, at: i32, tokens: &mut [Token]) {
//   let c = source;
//   // add_token(tokens, ttype)
// }

fn match_next(iter: &mut Peekable<Chars>, target: &char) -> bool {
  if let Some(c) = iter.peek() {
    return c == target
  }
  false
}

// fn advance() {
 
// }

fn add_token(line: i32, tokens: &mut Vec<Token>, ttype: TokenType) {
  tokens.push(Token {ttype, line});

}

