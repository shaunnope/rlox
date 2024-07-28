use super::*;
use crate::token::TokenType;

#[test]
fn example() {
  let expression = Expr::Binary {
      left: Box::new(Expr::Unary {
          op: Token {
              ttype: TokenType::Minus,
              line: 1
          },
          right: Box::new(Expr::Literal(
              Token {
                  ttype: TokenType::Number(123.0),
                  line: 1
              }
          ))
      }),
      op: Token {
          ttype: TokenType::Star,
          line: 1
      },
      right: Box::new(Expr::Grouping(
          Box::new(Expr::Literal(
              Token { 
                  ttype: TokenType::Number(45.67), 
                  line: 1 
              }
          ))
      )),
  };

  assert_eq!("(* (- 123) (group 45.67))", expression.to_string())
}