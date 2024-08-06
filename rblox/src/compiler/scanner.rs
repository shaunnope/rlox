#[cfg(test)]
mod tests;

use std::{iter::Peekable, str::CharIndices};

use crate::{
  common::Span,
  compiler::scanner::{
    identifier::{is_valid_identifier_start, is_valid_identifier_tail},
    error::ScanError,
    token::{Token, TokenType}
  }
};
 
pub mod token;
pub mod error;
pub mod identifier;

pub struct Scanner<'src> {
  src: &'src str,
  chars: Peekable<CharIndices<'src>>,
  current: (usize, char),
  lexeme_start: usize,
  line: u32,
  emitted_eof: bool,
}

/// Token iterator
impl Iterator for Scanner<'_> {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    if self.emitted_eof {
      return None;
    }
    let mut kind;

    use TokenType as TT;
    // Skip over util tokens
    loop {
      // Ensures the next token starts with a new span.
      self.lexeme_start = self.current.0;
      kind = self.scan_token();
      match kind {
        TT::Whitespace(_) => continue, 
        // TT::Comment(_) | TT::BlockComment(_) => continue,
        TT::Error(_) => break, // emit errors to be reported in compiler
        TT::Dummy => unreachable!("Source code should not contain dummy tokens."),
        _ => break
      }
    }

    if kind == TT::EOF {
      self.emitted_eof = true;
    }

    let span = match &kind {
      TT::BlockComment(_, line) => {
        let mut span = self.lex_span();
        span.2 = *line;
        span
      },
      _ => self.lex_span()
    };

    Some(Token {
      kind,
      span,
    })
  }
}

// The scanner implementation.
impl Scanner<'_> {
  /// Tries to scan the current character.
  fn scan_token(&mut self) -> TokenType {
    use TokenType::*;
    match self.advance() {
      '\0' => EOF,
      '(' => LeftParen,
      ')' => RightParen,
      '{' => LeftBrace,
      '}' => RightBrace,
      ';' => Semicolon,
      ',' => Comma,
      '.' => Dot,
      '!' => self.take_select('=', BangEqual, Bang),
      '=' => self.take_select('=', EqualEqual, Equal),
      '>' => self.take_select('=', GreaterEqual, Greater),
      '<' => self.take_select('=', LessEqual, Less),
      '+' => Plus,
      '-' => Minus,
      '*' => Star,
      '"' => self.string(),
      '/' => self.comment_or_slash(),
      c if c.is_ascii_digit() => self.number(),
      '\n' => {
        self.line += 1;
        self.whitespace()
      },
      c if c.is_ascii_whitespace() => self.whitespace(),
      c if is_valid_identifier_start(c) => self.identifier_or_keyword(),
      unexpected => Error(ScanError::UnexpectedChar(unexpected)),
    }
  }

  /// Tries to scan a string.
  fn string(&mut self) -> TokenType {
    self.consume_until('"');
    if self.is_at_end() {
      return TokenType::Error(ScanError::UnterminatedString);
    }
    self.advance(); // The closing `"`
    TokenType::String(self.lex(1, -1).into())
  }

  /// Tries to scan a comment or a slash.
  fn comment_or_slash(&mut self) -> TokenType {
    match self.current.1 {
      '/' => self.comment(),
      '*' => self.block_comment(),
      _ => TokenType::Slash,
    }
  }

  /// Scans a single line comment
  fn comment(&mut self) -> TokenType {
    self.consume_until('\n');

    TokenType::Comment(self.lex(2, 0).into())
  }

  /// Tries to scan a block comment
  fn block_comment(&mut self) -> TokenType {
    self.advance(); // consume first *
    let line = self.line;
    while !self.is_at_end() {
      match self.advance() {
        '*' => {
          if self.take('/') {
            // end of block
            break;
          }
        }
        '\n' => self.line += 1,
        _ => continue,
      }
    }
    if self.is_at_end() {
      return TokenType::Error(ScanError::UnterminatedComment);
    }
    TokenType::BlockComment(self.lex(2, -2).into(), line)
  }

  /// Tries to scan a number.
  fn number(&mut self) -> TokenType {
    while self.current.1.is_ascii_digit() {
      self.advance();
    }
    if self.current.1 == '.' && self.peek().1.is_ascii_digit() {
      self.advance(); // The `.` separator
      while self.current.1.is_ascii_digit() {
        self.advance();
      }
    }
    match self.lex(0, 0).parse() {
      Ok(parsed) => TokenType::Number(parsed),
      Err(_) => TokenType::Error(ScanError::InvalidNumberLiteral),
    }
  }

  /// Scans a newline or a whitespace.
  fn whitespace(&mut self) -> TokenType {
    while self.current.1.is_ascii_whitespace() {
      if self.current.1 == '\n' {
        self.line += 1;
      }
      self.advance();
    }
    TokenType::Whitespace(self.lex(0, 0).into())
  }

  /// Scans a keyword or an identifier.
  fn identifier_or_keyword(&mut self) -> TokenType {
    while is_valid_identifier_tail(self.current.1) {
      self.advance();
    }
    let name = self.lex(0, 0);
    if name == "NaN" {
      return TokenType::Number(f64::NAN);
    }
    TokenType::from(name)
  }
}

// The scanner helper methods.
impl<'src> Scanner<'src> {
  /// Creates a new scanner.
  pub fn new(src: &'src str) -> Self {
    let mut scanner = Self {
      src,
      chars: src.char_indices().peekable(),
      current: (0, '\0'),
      lexeme_start: 0,
      line: 1,
      emitted_eof: false,
    };
    scanner.advance(); // First advancement to set current char
    scanner
  }

  /// Peeks at the next character tuple.
  #[inline]
  fn peek(&mut self) -> (usize, char) {
    self
      .chars
      .peek()
      .unwrap_or(&(self.src.len(), '\0'))
      .to_owned()
  }

  /// Returns the current character and advances `current` cursor.
  #[inline]
  fn advance(&mut self) -> char {
    let curr = self.current.1;
    self.current = self.chars.next().unwrap_or((self.src.len(), '\0'));
    curr
  }

  /// Checks if the current character matches the given one. In such case advances and returns
  /// true. Otherwise returns false.
  #[inline]
  fn take(&mut self, expected: char) -> bool {
    if self.current.1 != expected {
      return false;
    }
    self.advance();
    true
  }

  /// Checks if the current character matches the given one. In such case, advances and returns
  /// `a`, otherwise returns `b`.
  #[inline]
  fn take_select<T>(&mut self, expected: char, a: T, b: T) -> T {
    match self.take(expected) {
      true => a,
      false => b,
    }
  }

  /// Returns the current lexeme span.
  #[inline]
  fn lex_span(&self) -> Span {
    Span::new(self.lexeme_start, self.current.0, self.line)
  }

  /// Returns a lexeme slice.
  #[inline]
  fn lex(&self, lo: isize, hi: isize) -> &'src str {
    let span = self.lex_span().updated(lo, hi);
    &self.src[span.0..span.1]
  }

  /// Checks if the scanner has finished.
  #[inline]
  fn is_at_end(&self) -> bool {
    self.current.1 == '\0'
  }

  /// Scans until before a matched character, or end of file
  fn consume_until(&mut self, ch: char) {
    while self.current.1 != ch && !self.is_at_end() {
      self.advance();
    }
  }
}
