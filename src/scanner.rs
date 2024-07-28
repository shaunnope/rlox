#[cfg(test)]
mod tests;

use std::str::Chars;
use itertools::{Itertools, MultiPeek};
use crate::token::{Token, TokenType};

pub fn scan_tokens(source: &str) -> Vec<Token> {

  let mut res = Vec::new();
  let tokens = &mut res;

  let mut line = 1;
  // let mut start = 0;

  let iter = &mut source.chars().multipeek();

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
          while let Some(_) = iter.next() {
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

      // string
      '"' => {
        let pos = line;
        let s = parse_string(&mut line, iter);

        add_token(pos, tokens, TokenType::String(s))
      },

      // number
      '0'..='9' => {
        if let Some(n) = parse_number(ch, iter) {
          add_token(line, tokens, TokenType::Number(n));
        } else {
          crate::error(line, "Failed to parse number")
        }
      }

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

fn match_next(iter: &mut MultiPeek<Chars>, target: &char) -> bool {
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

fn parse_string(line: &mut i32, iter: &mut MultiPeek<Chars>) -> String {
  iter
    .by_ref().take_while(
      |ch| match ch {
        '"' => false,
        '\n' => {
          *line += 1;
          true
        }
        _ => true
      }
    ).collect()
}

fn parse_number(start: char, iter: &mut MultiPeek<Chars>) -> Option<f64> {
  let mut fractional = false;
  let mut tail = vec![];

  while let Some(ch) = iter.peek() {
    match ch {
      '0'..='9' => {
        tail.push(*ch);
        iter.next();
      },
      '.' => {
        fractional = true;
        break;
      },
      _ => break,
    }
  }
  if !fractional {
    return build_number(start, tail);
  }

  if let Some(ch) = iter.peek() {
    match ch {
      '0'..='9' => { // a valid decimal point
        tail.push('.');
        tail.push(*ch);
        iter.next();
        iter.next();
      },
      _ => { // not a decimal point. number complete
        return build_number(start, tail);
      }
    }
    // continue parsing number
    while let Some(ch) = iter.peek() {
      match ch {
        '0'..='9' => {
          tail.push(*ch);
          iter.next();
        },
        _ => break,
      }
    }
    return build_number(start, tail);
  }

  return build_number(start, tail);
}

fn build_number(start: char, tail: Vec<char>) -> Option<f64> {
  let tail: String = tail.into_iter().collect();

  match (String::from(start) + &tail).parse() {
    Ok(n) => Some(n),
    Err(_) => None
  }
}