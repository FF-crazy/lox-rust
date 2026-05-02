
use crate::{object::Object, token::Token};

#[derive(Debug)]
pub enum Expr<'src> {
  Binary {
    left: Box<Expr<'src>>,
    operator: Token<'src>,
    right: Box<Expr<'src>>,
  },
  Grouping(Box<Expr<'src>>),
  Literal(Object),
  Unary {
    operator: Token<'src>,
    right: Box<Expr<'src>>,
  },
}

#[cfg(test)]
mod test {
  use crate::{
    object::Object,
    token::{Token, TokenType},
  };
  use super::*;

  #[test]
  fn test() {
    let _expr = Expr::Unary {
      operator: Token::new(TokenType::Minus, "-", None, 1),
      right: Box::new(Expr::Literal(Object::Number(123_f64))),
    };
  }
}
