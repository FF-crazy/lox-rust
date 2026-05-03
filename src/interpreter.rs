use crate::{
  error_handling::RuntimeError,
  expr::Expr,
  object::Object,
  token::{Token, TokenType},
};

pub struct Interpreter {}

impl<'src> Interpreter {
  pub fn new() -> Interpreter {
    Interpreter {}
  }

  pub fn execute(&self, expr: &Expr<'src>) -> Result<Object, RuntimeError<'src>> {
    Self::evaluate(expr)
  }

  fn evaluate(expr: &Expr<'src>) -> Result<Object, RuntimeError<'src>> {
    match expr {
      Expr::Literal(value) => Ok(value.clone()),
      Expr::Grouping(inner) => Self::evaluate(inner),
      Expr::Unary { operator, right } => {
        let right_value = Self::evaluate(right)?;
        Self::apply_unary(operator, right_value)
      }
      Expr::Binary {
        left,
        operator,
        right,
      } => {
        let left_value = Self::evaluate(left)?;
        let right_value = Self::evaluate(right)?;
        Self::apply_binary(left_value, operator, right_value)
      }
    }
  }

  fn apply_unary(
    operator: &Token<'src>,
    right_value: Object,
  ) -> Result<Object, RuntimeError<'src>> {
    match operator.ttype() {
      TokenType::Bang => Ok(Self::object_bang(right_value)),
      TokenType::Minus => Self::object_minus(right_value, operator),
      _ => Err(RuntimeError {
        message: "Never run this line".to_string(),
        token: operator.clone(),
      }),
    }
  }

  fn object_minus(
    right_value: Object,
    operator: &Token<'src>,
  ) -> Result<Object, RuntimeError<'src>> {
    match right_value {
      Object::Number(value) => Ok(Object::Number(-value)),
      _ => Err(RuntimeError {
        message: String::from("Operand must be a Number"),
        token: operator.clone(),
      }),
    }
  }

  fn object_bang(right_value: Object) -> Object {
    match right_value {
      Object::Number(_) | Object::String(_) => Object::Boolean(false),
      Object::Nil => Object::Boolean(true),
      Object::Boolean(flag) => Object::Boolean(!flag),
    }
  }

  fn apply_binary(
    left_value: Object,
    operator: &Token<'src>,
    right_value: Object,
  ) -> Result<Object, RuntimeError<'src>> {
    match operator.ttype() {
      TokenType::Plus => Self::object_plus(left_value, right_value, operator),
      TokenType::Minus => match (left_value, right_value) {
        (Object::Number(l), Object::Number(r)) => Ok(Object::Number(l - r)),
        _ => Err(RuntimeError {
          message: "'-' operator must be used between Numbers".to_string(),
          token: operator.clone(),
        }),
      },
      TokenType::Star => match (left_value, right_value) {
        (Object::Number(l), Object::Number(r)) => Ok(Object::Number(l * r)),
        _ => Err(RuntimeError {
          message: "'*' operator must be used between Numbers".to_string(),
          token: operator.clone(),
        }),
      },
      TokenType::Slash => match (left_value, right_value) {
        (Object::Number(l), Object::Number(r)) => Ok(Object::Number(l / r)),
        _ => Err(RuntimeError {
          message: "'/' operator must be used between Numbers".to_string(),
          token: operator.clone(),
        }),
      },
      TokenType::Equal => Ok(Object::Boolean(Self::object_equal(left_value, right_value))),
      TokenType::BangEqual => Ok(Object::Boolean(!Self::object_equal(
        left_value,
        right_value,
      ))),
      TokenType::Greater => Self::object_compare(left_value, right_value, operator, |x, y| x > y),
      TokenType::Less => Self::object_compare(left_value, right_value, operator, |x, y| x < y),
      TokenType::GreaterEqual => Self::object_compare(right_value, left_value, operator, |x, y| x >= y),
      TokenType::LessEqual => Self::object_compare(right_value, left_value, operator, |x, y| x <= y),
      _ => Err(RuntimeError {
        message: "Never run this line".to_string(),
        token: operator.clone(),
      }),
    }
  }

  fn object_plus(
    left: Object,
    right: Object,
    token: &Token<'src>,
  ) -> Result<Object, RuntimeError<'src>> {
    match (left, right) {
      (Object::Number(l), Object::Number(r)) => Ok(Object::Number(l + r)),
      (Object::String(l), Object::String(r)) => Ok(Object::String(l + &r)),
      _ => Err(RuntimeError {
        message: "Operand must be Number or String".to_string(),
        token: token.clone(),
      }),
    }
  }

  fn object_compare(
    left: Object,
    right: Object,
    token: &Token<'src>,
    cmp: impl Fn(f64, f64) -> bool,
  ) -> Result<Object, RuntimeError<'src>> {
    match (left, right) {
      (Object::Number(l), Object::Number(r)) => Ok(Object::Boolean(cmp(l, r))),
      _ => Err(RuntimeError {
        message: "Operands must be Numbers".to_string(),
        token: token.clone(),
      }),
    }
  }

  fn object_equal(left: Object, right: Object) -> bool {
    match (left, right) {
      (Object::Number(l), Object::Number(r)) => l == r,
      (Object::Boolean(l), Object::Boolean(r)) => l == r,
      (Object::String(l), Object::String(r)) => l == r,
      (Object::Nil, Object::Nil) => true,
      _ => false,
    }
  }
}

#[cfg(test)]
mod test {
  use crate::{parser::Parser, scanner::Scanner};

  use super::*;

  fn get(s: &str) -> Object {
    let scan = Scanner::new(s);
    let tokens = scan.scan_tokens().unwrap();
    let parser = Parser::new(tokens);
    let expr = parser.parse().unwrap();
    let i = Interpreter::new();
    i.execute(&expr).unwrap()
  }

  #[test]
  fn test() {
    assert_eq!(Object::Boolean(false), get("1 >= 2"));
    assert_eq!(Object::Boolean(true), get("3 >= 3"));
  }
}
