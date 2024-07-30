// #[cfg(test)]
// mod tests;

use crate::{
  parser::scanner::error::ScanError,
  span::Span,
  token::{Token, TokenType},
  // error::{Error, LoxError, Type}
};

pub mod error;

pub struct Scanner<'src> {
  src: &'src str,
  chars: Vec<(usize, char)>, // Start byte index and char
  cursor: usize,
  lex_span_start: usize,
  emitted_eof: bool,
}

impl Iterator for Scanner<'_> {
  type Item = Token;

  fn next(&mut self) -> Option<Self::Item> {
    if self.emitted_eof {
      return None;
    }
    // Ensures the next token starts with a new span.
    self.lex_span_start = self.peek(0).0;
    let kind = self.scan_token();
    if kind == TokenType::EOF {
      self.emitted_eof = true;
    }
    Some(Token {
      kind,
      span: self.lex_span(),
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
      c if c.is_ascii_whitespace() => self.whitespace(),
      // c if is_valid_identifier_start(c) => self.identifier_or_keyword(),
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
    match self.peek(0).1 {
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

    while !self.is_at_end() {
      match self.advance() {
        '*' => {
          if self.take('/') {
            // end of block
            break;
          }
        }
        _ => continue,
      }
    }
    if self.is_at_end() {
      return TokenType::Error(ScanError::UnterminatedComment);
    }
    self.advance(); // The closing `/`
    TokenType::BlockComment(self.lex(2, 0).into())
  }

  /// Tries to scan a number.
  fn number(&mut self) -> TokenType {
    while self.current().is_ascii_digit() {
      self.advance();
    }
    if self.current() == '.' && self.peek(1).1.is_ascii_digit() {
      self.advance(); // The `.` separator
      while self.current().is_ascii_digit() {
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
    while self.current().is_ascii_whitespace() {
      self.advance();
    }
    TokenType::Whitespace(self.lex(0, 0).into())
  }
}

// The scanner helper methods.
impl<'src> Scanner<'src> {
  /// Creates a new scanner.
  pub fn new(src: &'src str) -> Self {
    Self {
      src,
      chars: src.char_indices().collect(),
      cursor: 0,
      lex_span_start: 0,
      emitted_eof: false,
    }
  }

  /// Peeks a character tuple with the given offset from the cursor.
  #[inline]
  fn peek(&self, offset: isize) -> (usize, char) {
    self
      .chars
      .get((self.cursor as isize + offset) as usize)
      .copied()
      .unwrap_or((self.src.len(), '\0'))
  }

  /// Peeks into the current character (not yet consumed).
  #[inline]
  fn current(&self) -> char {
    self.peek(0).1
  }

  /// Returns the current character and advances the `current` cursor.
  #[inline]
  fn advance(&mut self) -> char {
    self.cursor += 1;
    self.peek(-1).1
  }

  /// Checks if the current character matches the given one. In such case advances and returns
  /// true. Otherwise returns false.
  #[inline]
  fn take(&mut self, expected: char) -> bool {
    if self.current() != expected {
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
    Span::new(self.lex_span_start, self.peek(0).0)
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
    self.cursor >= self.chars.len()
  }

  /// Scans until before a matched character, or end of file
  fn consume_until(&mut self, ch: char) {
    while self.current() != ch && !self.is_at_end() {
      self.advance();
    }
  }
}
