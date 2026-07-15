use crate::{object::Object, token::Token};
use std::fmt;

#[derive(Debug, Clone)]
pub enum Expr<'src> {
  Binary {
    left: Box<Expr<'src>>,
    operator: Token<'src>,
    right: Box<Expr<'src>>,
  },
  Grouping(Box<Expr<'src>>),
  Literal(Object<'src>),
  Unary {
    operator: Token<'src>,
    right: Box<Expr<'src>>,
  },
  Variable(Token<'src>),
  Assign {
    name: Token<'src>,
    value: Box<Expr<'src>>,
  },
  Logical {
    left: Box<Expr<'src>>,
    operator: Token<'src>,
    right: Box<Expr<'src>>,
  },
  Call {
    callee: Box<Expr<'src>>,
    paren: Token<'src>,
    arguments: Vec<Expr<'src>>,
  }
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
      Expr::Variable(var) => write!(f, "{}", var),
      Expr::Assign { name, value } => write!(f, "{} = {}", name, value),
      Expr::Logical {
        left,
        operator,
        right,
      } => {
        write!(f, "({} {} {})", operator.lexeme(), left, right)
      },
      Expr::Call { callee, arguments, .. } => {
        write!(f, "(call {}", callee)?;
        for argument in arguments {
          write!(f, " {}", argument)?;
        }
        write!(f, ")")
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
    let expr = Expr::Unary {
      operator: Token::new(TokenType::Minus, "-", None, 1),
      right: Box::new(Expr::Literal(Object::Number(123_f64))),
    };
    println!("{}", expr);
  }
}
