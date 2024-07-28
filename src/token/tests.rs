use super::*;

#[test]
fn correct_token_representations() {
  let token = Token {ttype: TokenType::LeftParen, line: 0};

  assert_eq!("LeftParen 0", format!("{}", token), "Incorrect repr for LeftBrace")
}