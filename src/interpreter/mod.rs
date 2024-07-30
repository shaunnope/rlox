
use crate::ast;

mod expr;

pub fn interpret(expression: ast::Expr) {
  match expr::evaluate(&expression) {
    Ok(None) => println!("nil"),
    Ok(Some(res)) => {
      if let Ok(_num) = expr::number(&Some(res)) {

      }
      // println!("{}", res);
    },
    Err(_err) => {}
  }
}
