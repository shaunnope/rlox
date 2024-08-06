use super::*;

#[test]
fn emits_correct_tokens() {
  let source = "( )  {} ,.-+;
*  / ! != = == > >= < <=
asdf \"asdf\" 12 3.4 \"0.1\" 
and class else false fun for if nil or
print return super this true var while // comment
/* block
comment */
/* inline block*/
forest varied\0";

  let mut scanner = Scanner::new(source);

  assert_eq!(scanner.next(), Some(Token::new(TokenType::LeftParen, Span::new(0, 1, 1))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::RightParen, Span::new(2, 3, 1))));

  assert_eq!(scanner.next(), Some(Token::new(TokenType::LeftBrace, Span::new(5, 6, 1))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::RightBrace, Span::new(6, 7, 1))));

  assert_eq!(scanner.next(), Some(Token::new(TokenType::Comma, Span::new(8, 9, 1))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Dot, Span::new(9, 10, 1))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Minus, Span::new(10, 11, 1))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Plus, Span::new(11, 12, 1))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Semicolon, Span::new(12, 13, 1))));

  assert_eq!(scanner.next(), Some(Token::new(TokenType::Star, Span::new(14, 15, 2))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Slash, Span::new(17, 18, 2))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Bang, Span::new(19, 20, 2))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::BangEqual, Span::new(21, 23, 2))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Equal, Span::new(24, 25, 2))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::EqualEqual, Span::new(26, 28, 2))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Greater, Span::new(29, 30, 2))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::GreaterEqual, Span::new(31, 33, 2))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Less, Span::new(34, 35, 2))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::LessEqual, Span::new(36, 38, 2))));

  assert_eq!(scanner.next(), Some(Token::new(TokenType::Identifier("asdf".into()), Span::new(39, 43, 3))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::String("asdf".into()), Span::new(44, 50, 3))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Number(12.0), Span::new(51, 53, 3))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Number(3.4), Span::new(54, 57, 3))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::String("0.1".into()), Span::new(58, 63, 3))));

  assert_eq!(scanner.next(), Some(Token::new(TokenType::And, Span::new(65, 68, 4))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Class, Span::new(69, 74, 4))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Else, Span::new(75, 79, 4))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::False, Span::new(80, 85, 4))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Fun, Span::new(86, 89, 4))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::For, Span::new(90, 93, 4))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::If, Span::new(94, 96, 4))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Nil, Span::new(97, 100, 4))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Or, Span::new(101, 103, 4))));

  assert_eq!(scanner.next(), Some(Token::new(TokenType::Print, Span::new(104, 109, 5))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Return, Span::new(110, 116, 5))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Super, Span::new(117, 122, 5))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::This, Span::new(123, 127, 5))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::True, Span::new(128, 132, 5))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Var, Span::new(133, 136, 5))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::While, Span::new(137, 142, 5))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Comment(" comment".into()), Span::new(143, 153, 5))));

  assert_eq!(scanner.next(), Some(Token::new(TokenType::BlockComment(" block\ncomment ".into(), 6), Span::new(154, 173, 6))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::BlockComment(" inline block".into(), 8), Span::new(174, 191, 8))));

  assert_eq!(scanner.next(), Some(Token::new(TokenType::Identifier("forest".into()), Span::new(192, 198, 9))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::Identifier("varied".into()), Span::new(199, 205, 9))));
  assert_eq!(scanner.next(), Some(Token::new(TokenType::EOF, Span::new(205, 206, 9))));

}
