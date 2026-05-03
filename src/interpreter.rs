use crate::{
  error_handling::RuntimeError,
  expr::Expr,
  object::Object,
  token::{Token, TokenType},
};

pub struct Interpreter<'src> {
  expr: Expr<'src>,
}

impl<'src> Interpreter<'src> {
  pub fn new(expr: Expr<'src>) -> Interpreter<'src> {
    Interpreter { expr }
  }

  pub fn execute(&self) -> Result<Object, RuntimeError<'src>> {
    Self::evaluate(&self.expr)
  }

  fn evaluate(expr: &Expr<'src>) -> Result<Object, RuntimeError<'src>> {
    match expr {
      Expr::Literal(value) => Ok(value.clone()),
      Expr::Grouping(inner) => Self::evaluate(&*inner),
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
      TokenType::Bang => Self::object_bang(right_value, operator),
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

  fn object_bang(
    right_value: Object,
    operator: &Token<'src>,
  ) -> Result<Object, RuntimeError<'src>> {
    match right_value {
      Object::Number(_) | Object::String(_) => Ok(Object::Boolean(false)),
      Object::Nil => Ok(Object::Boolean(true)),
      Object::Boolean(flag) => Ok(Object::Boolean(!flag)),
    }
  }

  fn apply_binary(
    left_value: Object,
    operator: &Token<'src>,
    right_value: Object,
  ) -> Result<Object, RuntimeError<'src>> {
    match operator.ttype() {
      TokenType::Plus => Self::object_plus(left_value, right_value, operator),
      TokenType::Minus => {
        let right_value = Self::object_minus(right_value, operator)?;
        Self::object_plus(left_value, right_value, operator)
      }
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
          message: "'*' operator must be used between Numbers".to_string(),
          token: operator.clone(),
        }),
      },
      TokenType::Equal => Ok(Self::object_equal(left_value, right_value)),
      TokenType::BangEqual => {
        let object = Self::object_equal(left_value, right_value);
        if let Object::Boolean(flag)  = object {
          Ok(Object::Boolean(!flag))
        } else {
          Err(RuntimeError { message: "Never run this line".to_string(), token: operator.clone() })
        }
      }
      TokenType::Greater => Self::object_greater(left_value, right_value, operator),
      TokenType::Less => Self::object_less(left_value, right_value, operator),
      TokenType::GreaterEqual => Self::object_less(right_value, left_value, operator),
      TokenType::LessEqual => Self::object_greater(right_value, left_value, operator),
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

  fn object_greater(
    left: Object,
    right: Object,
    token: &Token<'src>,
  ) -> Result<Object, RuntimeError<'src>> {
    match (left, right) {
      (Object::Number(l), Object::Number(r)) => Ok(Object::Boolean(l > r)),
      _ => Err(RuntimeError {
        message: "Operand must be Numbers".to_string(),
        token: token.clone(),
      }),
    }
  }

  fn object_less(
    left: Object,
    right: Object,
    token: &Token<'src>,
  ) -> Result<Object, RuntimeError<'src>> {
    match (left, right) {
      (Object::Number(l), Object::Number(r)) => Ok(Object::Boolean(l < r)),
      _ => Err(RuntimeError {
        message: "Operand must be Numbers".to_string(),
        token: token.clone(),
      }),
    }
  }

  fn object_equal(left: Object, right: Object) -> Object {
    match (left, right) {
      (Object::Number(l), Object::Number(r)) => Object::Boolean(l == r),
      (Object::Boolean(l), Object::Boolean(r)) => Object::Boolean(l == r),
      (Object::String(l), Object::String(r)) => Object::Boolean(l == r),
      _ => Object::Boolean(false),
    }
  }
}
