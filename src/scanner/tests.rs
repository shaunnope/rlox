
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
    ], scan_tokens(source)?);
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
    ], scan_tokens(source)?);
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
    ], scan_tokens(source)?);
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
    ], scan_tokens(source)?);
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
    ], scan_tokens(source)?);
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
    ], scan_tokens(source)?);
}

#[test]
fn correct_identifiers() {
  let source = 
  "asd a012s_.
  ns_+0 asm.di4";

  assert_eq!(vec![
    Token {ttype: TokenType::Identifier(String::from("asd")), line: 1},
    Token {ttype: TokenType::Identifier(String::from("a012s_")), line: 1},
    Token {ttype: TokenType::Dot, line: 1},
    Token {ttype: TokenType::Identifier(String::from("ns_")), line: 2},
    Token {ttype: TokenType::Plus, line: 2},
    Token {ttype: TokenType::Number(0.0), line: 2},
    Token {ttype: TokenType::Identifier(String::from("asm")), line: 2},
    Token {ttype: TokenType::Dot, line: 2},
    Token {ttype: TokenType::Identifier(String::from("di4")), line: 2},
    Token {ttype: TokenType::EOF, line: 2},
    ], scan_tokens(source)?);
}

#[test]
fn correct_reserved() {
  let source = 
  "and class else false fun for
  if nil or print return super this true
  var while forest andclass For";

  assert_eq!(vec![
    Token {ttype: TokenType::And, line: 1},
    Token {ttype: TokenType::Class, line: 1},
    Token {ttype: TokenType::Else, line: 1},
    Token {ttype: TokenType::False, line: 1},
    Token {ttype: TokenType::Fun, line: 1},
    Token {ttype: TokenType::For, line: 1},

    Token {ttype: TokenType::If, line: 2},
    Token {ttype: TokenType::Nil, line: 2},
    Token {ttype: TokenType::Or, line: 2},
    Token {ttype: TokenType::Print, line: 2},
    Token {ttype: TokenType::Return, line: 2},
    Token {ttype: TokenType::Super, line: 2},
    Token {ttype: TokenType::This, line: 2},
    Token {ttype: TokenType::True, line: 2},

    Token {ttype: TokenType::Var, line: 3},
    Token {ttype: TokenType::While, line: 3},
    Token {ttype: TokenType::Identifier(String::from("forest")), line: 3},
    Token {ttype: TokenType::Identifier(String::from("andclass")), line: 3},
    Token {ttype: TokenType::Identifier(String::from("For")), line: 3},
    Token {ttype: TokenType::EOF, line: 3},
    ], scan_tokens(source)?);
}