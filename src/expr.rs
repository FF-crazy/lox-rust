use crate::{object::Object, token::Token};
use std::fmt;

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

impl<'src> fmt::Display for Expr<'src> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Expr::Literal(value) => write!(f, "{}", value),
      Expr::Grouping(inner) => write!(f, "(group {})", inner),
      Expr::Unary { operator, right } => write!(f, "({} {})", operator.lexeme(), right),
      Expr::Binary {
        left,
        operator,
        right,
      } => {
        write!(f, "({} {} {})", operator.lexeme(), left, right)
      }
    }
  }
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::{
    object::Object,
    token::{Token, TokenType},
  };

  #[test]
  fn test() {
    let _expr = Expr::Unary {
      operator: Token::new(TokenType::Minus, "-", None, 1),
      right: Box::new(Expr::Literal(Object::Number(123_f64))),
    };
  }
}
