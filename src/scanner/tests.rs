
use super::*;

#[ignore]
#[test]
fn comments_ignored() {
  let source = 
  "(( // asdf

  //comment +3
  *+- .";

  assert_eq!(vec![
    Token {ttype: TokenType::LeftParen, line: 1},
    Token {ttype: TokenType::LeftParen, line: 1},
    Token {ttype: TokenType::Star, line: 4},
    Token {ttype: TokenType::Plus, line: 4},
    Token {ttype: TokenType::Minus, line: 4},
    Token {ttype: TokenType::Dot, line: 4},
    Token {ttype: TokenType::EOF, line: 4},
    ], scan_tokens(source));
}


#[ignore]
#[test]
fn correct_single_char_tokens() {
  let source = 
  "(( )){}
  *+- .;";

  assert_eq!(vec![
    Token {ttype: TokenType::LeftParen, line: 1},
    Token {ttype: TokenType::LeftParen, line: 1},
    Token {ttype: TokenType::RightParen, line: 1},
    Token {ttype: TokenType::RightParen, line: 1},
    Token {ttype: TokenType::LeftBrace, line: 1},
    Token {ttype: TokenType::RightBrace, line: 1},
    Token {ttype: TokenType::Star, line: 2},
    Token {ttype: TokenType::Plus, line: 2},
    Token {ttype: TokenType::Minus, line: 2},
    Token {ttype: TokenType::Dot, line: 2},
    Token {ttype: TokenType::Semicolon, line: 2},
    Token {ttype: TokenType::EOF, line: 2},
    ], scan_tokens(source));
}

#[ignore]
#[test]
fn correct_space_delimited_variable_length_tokens() {
  let source = 
  "! != 
  < >
  <= >=
  = ==";

  assert_eq!(vec![
    Token {ttype: TokenType::Bang, line: 1},
    Token {ttype: TokenType::BangEqual, line: 1},
    Token {ttype: TokenType::Less, line: 2},
    Token {ttype: TokenType::Greater, line: 2},
    Token {ttype: TokenType::LessEqual, line: 3},
    Token {ttype: TokenType::GreaterEqual, line: 3},
    Token {ttype: TokenType::Equal, line: 4},
    Token {ttype: TokenType::EqualEqual, line: 4},
    Token {ttype: TokenType::EOF, line: 4},
    ], scan_tokens(source));
}

#[ignore]
#[test]
fn correct_one_lookahead() {
  let source = 
  "!+!=+
  <.>.
  <=(>=)
  ={==}";

  assert_eq!(vec![
    Token {ttype: TokenType::Bang, line: 1},
    Token {ttype: TokenType::Plus, line: 1},
    Token {ttype: TokenType::BangEqual, line: 1},
    Token {ttype: TokenType::Plus, line: 1},
    Token {ttype: TokenType::Less, line: 2},
    Token {ttype: TokenType::Dot, line: 2},
    Token {ttype: TokenType::Greater, line: 2},
    Token {ttype: TokenType::Dot, line: 2},
    Token {ttype: TokenType::LessEqual, line: 3},
    Token {ttype: TokenType::LeftParen, line: 3},
    Token {ttype: TokenType::GreaterEqual, line: 3},
    Token {ttype: TokenType::RightParen, line: 3},
    Token {ttype: TokenType::Equal, line: 4},
    Token {ttype: TokenType::LeftBrace, line: 4},
    Token {ttype: TokenType::EqualEqual, line: 4},
    Token {ttype: TokenType::RightBrace, line: 4},
    Token {ttype: TokenType::EOF, line: 4},
    ], scan_tokens(source));
}

#[ignore]
#[test]
fn correct_strings() {
  let source = 
  ".\"asdk+\".
  \"lorem ipsum
  asdf=
  \"";

  assert_eq!(vec![
    Token {ttype: TokenType::Dot, line: 1},
    Token {ttype: TokenType::String(String::from("asdk+")), line: 1},
    Token {ttype: TokenType::Dot, line: 1},
    Token {ttype: TokenType::String(
      String::from("lorem ipsum\n  asdf=\n  ")
    ), line: 2},
    Token {ttype: TokenType::EOF, line: 4},
    ], scan_tokens(source));
}

#[test]
fn correct_numbers() {
  let source = 
  "0 12 3.4 5+
  .23 4.5. 9.";

  assert_eq!(vec![
    Token {ttype: TokenType::Number(0.0), line: 1},
    Token {ttype: TokenType::Number(12.0), line: 1},
    Token {ttype: TokenType::Number(3.4), line: 1},
    Token {ttype: TokenType::Number(5.0), line: 1},
    Token {ttype: TokenType::Plus, line: 1},
    Token {ttype: TokenType::Dot, line: 2},
    Token {ttype: TokenType::Number(23.0), line: 2},
    Token {ttype: TokenType::Number(4.5), line: 2},
    Token {ttype: TokenType::Dot, line: 2},
    Token {ttype: TokenType::Number(9.0), line: 2},
    Token {ttype: TokenType::Dot, line: 2},
    Token {ttype: TokenType::EOF, line: 2},
    ], scan_tokens(source));
}