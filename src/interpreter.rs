use crate::{
  error_handling::RuntimeError,
  expr::Expr,
  object::Object,
  stmt::Stmt,
  token::{Token, TokenType},
};

use std::cmp::Ordering;
use std::collections::HashMap;

pub struct Interpreter {
  environment: Environment,
}

struct Environment {
  values: HashMap<String, Option<Object>>,
}

impl Environment {
  pub fn new() -> Environment {
    Environment {
      values: HashMap::new(),
    }
  }

  pub fn define_variable(&mut self, name: String, obj: Option<Object>) {
    self.values.insert(name, obj);
  }

  pub fn read_variable(&self, name: &String) -> Option<Option<Object>> {
    self.values.get(name).cloned()
  }
}

impl<'src> Interpreter {
  pub fn new() -> Interpreter {
    Interpreter {
      environment: Environment::new(),
    }
  }

  pub fn interpret(&mut self, stmts: &Vec<Stmt<'src>>) -> Result<(), RuntimeError<'src>> {
    for stmt in stmts {
      self.execute_statement(stmt)?;
    }
    Ok(())
  }

  fn execute_statement(&mut self, stmt: &Stmt<'src>) -> Result<(), RuntimeError<'src>> {
    match stmt {
      Stmt::Expression(expr) => {
        self.evaluate(expr)?;
        Ok(())
      }
      Stmt::Print(expr) => {
        let value = self.evaluate(expr)?;
        println!("{}", value);
        Ok(())
      }
      Stmt::Var { name, initializer } => {
        if let Some(expr) = initializer {
          let res = self.evaluate(expr)?;
          self
            .environment
            .define_variable(name.lexeme().to_string(), Some(res));
        } else {
          self
            .environment
            .define_variable(name.lexeme().to_string(), None);
        }
        Ok(())
      }
    }
  }

  fn evaluate(&mut self, expr: &Expr<'src>) -> Result<Object, RuntimeError<'src>> {
    match expr {
      Expr::Literal(value) => Ok(value.clone()),
      Expr::Grouping(inner) => self.evaluate(inner),
      Expr::Unary { operator, right } => {
        let right_value = self.evaluate(right)?;
        Self::apply_unary(operator, right_value)
      }
      Expr::Binary {
        left,
        operator,
        right,
      } => {
        let left_value = self.evaluate(left)?;
        let right_value = self.evaluate(right)?;
        Self::apply_binary(left_value, operator, right_value)
      }
      Expr::Variable(name) => {
        if let Some(var) = self.environment.read_variable(&name.lexeme().to_string()) {
          var.ok_or(RuntimeError {
            message: format!(
              "Variable '{}' is used before assign it a value",
              name.lexeme()
            ),
            token: name.clone(),
          })
        } else {
          Err(RuntimeError {
            message: format!("Undefined variable '{}'", name.lexeme()),
            token: name.clone(),
          })
        }
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
      TokenType::Star => Self::object_multiply(left_value, right_value, operator),
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
      TokenType::Greater => Self::object_compare(left_value, right_value, operator, |o| {
        o == Ordering::Greater
      }),
      TokenType::Less => {
        Self::object_compare(left_value, right_value, operator, |o| o == Ordering::Less)
      }
      TokenType::GreaterEqual => Self::object_compare(left_value, right_value, operator, |o| {
        matches!(o, Ordering::Equal | Ordering::Greater)
      }),
      TokenType::LessEqual => Self::object_compare(left_value, right_value, operator, |o| {
        matches!(o, Ordering::Equal | Ordering::Less)
      }),
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
        message: "'+' Operand must be Number or String".to_string(),
        token: token.clone(),
      }),
    }
  }

  fn object_multiply(
    left: Object,
    right: Object,
    token: &Token<'src>,
  ) -> Result<Object, RuntimeError<'src>> {
    match (left, right) {
      (Object::Number(l), Object::Number(r)) => Ok(Object::Number(l * r)),
      (Object::String(s), Object::Number(num)) | (Object::Number(num), Object::String(s)) => {
        if let Some(num) = Self::as_repetition_count(num) {
          Ok(Object::String(s.repeat(num)))
        } else {
          Err(RuntimeError {
            message: "String repetition count must be a non-negative integer".to_string(),
            token: token.clone(),
          })
        }
      }
      _ => Err(RuntimeError {
        message: format!("{} operator must be Number or String", token.lexeme()),
        token: token.clone(),
      }),
    }
  }

  fn as_repetition_count(n: f64) -> Option<usize> {
    if n.is_finite() && n >= 0.0 && n.fract() == 0.0 {
      Some(n as usize)
    } else {
      None
    }
  }

  fn object_compare(
    left: Object,
    right: Object,
    token: &Token<'src>,
    cmp: impl Fn(Ordering) -> bool,
  ) -> Result<Object, RuntimeError<'src>> {
    let ordering = match (&left, &right) {
      (Object::Number(l), Object::Number(r)) => l.partial_cmp(r),
      (Object::String(l), Object::String(r)) => l.partial_cmp(r),
      _ => {
        return Err(RuntimeError {
          message: "Operands must be both Numbers or both Strings".to_string(),
          token: token.clone(),
        });
      }
    };
    match ordering {
      Some(o) => Ok(Object::Boolean(cmp(o))),
      None => Err(RuntimeError {
        // NaN 等无法比较的情况
        message: "Operands are not comparable".to_string(),
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

  fn get(s: &str) {
    let scan = Scanner::new(s);
    let tokens = scan.scan_tokens().unwrap();
    let parser = Parser::new(tokens);
    let stmts = parser.parse().unwrap();
    let mut i = Interpreter::new();
    i.interpret(&stmts);
  }

  #[test]
  fn test() {}
}
