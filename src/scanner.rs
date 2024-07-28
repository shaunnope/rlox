#[cfg(test)]
mod tests;

use std::str::Chars;
use itertools::{Itertools, MultiPeek};
use crate::token::{Token, TokenType};

use crate::error::Error;

pub fn scan_tokens(source: &str) -> Result<Vec<Token>, Error> {

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
        if let Some(c) = iter.peek() {
          match c {
            '/' => consume_comment(iter), // single line comment
            '*' => consume_block_comment(&mut line, iter), // block comment
            _ => add_token(line, tokens, TokenType::Slash) // div operator
          }
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

      // identifiers
      'a'..='z'|'A'..='Z'|'_' => {
        add_token(line, tokens, parse_identifier(ch, iter));
      }

      _ => crate::error(line, "Unexpected character")
    };
    
  }

  tokens.push(
    Token { 
      ttype: TokenType::EOF, 
      line 
    });

  return Ok(res)
}


fn match_next(iter: &mut MultiPeek<Chars>, target: &char) -> bool {
  if let Some(c) = iter.peek() {
    return c == target
  }
  false
}

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

fn parse_identifier(start: char, iter: &mut MultiPeek<Chars>) -> TokenType {

  let tail: String = iter.peeking_take_while(
    |ch| {
    match ch {
      '0'..='9'|'a'..='z'|'A'..='Z'|'_' => true,
      _ => false,
    }
  }).collect();

  get_token_type(String::from(start) + &tail)
}

fn get_token_type(lexeme: String) -> TokenType {
  match lexeme.as_str() {
    "and" => TokenType::And,
    "class" => TokenType::Class,
    "else" => TokenType::Else,
    "false" => TokenType::False,
    "fun" => TokenType::Fun,
    "for" => TokenType::For,
    "if" => TokenType::If,
    "nil" => TokenType::Nil,
    "or" => TokenType::Or,
    "print" => TokenType::Print,
    "return" => TokenType::Return,
    "super" => TokenType::Super,
    "this" => TokenType::This,
    "true" => TokenType::True,
    "var" => TokenType::Var,
    "while" => TokenType::While,
    _ => TokenType::Identifier(lexeme)
  }
}

fn consume_comment(iter: &mut MultiPeek<Chars>) {
  while let Some(_) = iter.next() {
    if match_next(iter, &'\n') {
      break
    }
  }
}

fn consume_block_comment(line: &mut i32, iter: &mut MultiPeek<Chars>) {
  let pos = *line;
  iter.next(); // consume first *
  while let Some(ch) = iter.next() {
    match ch {
      '*' => {
        if match_next(iter, &'/') { // end of block
          iter.next();
          return;
        }
      },
      '\n' => { // inc line
        *line += 1;
      }
      _ => continue,
    }
  }

  // comment reached end of file
  crate::error(pos, "Block comment not closed")
}