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
  scopes: Vec<HashMap<String, Option<Object>>>,
}

impl Environment {
  pub fn new() -> Environment {
    Environment {
      scopes: vec![HashMap::new()],
    }
  }

  pub fn define_variable(&mut self, name: &str, obj: Option<Object>) -> Result<(), String> {
    let values = self
      .scopes
      .last_mut()
      .expect("Always have a static variable");
    if values.contains_key(name) {
      Err(format!("Variable '{}' is already declared in this scope ", name))
    } else {
      values.insert(name.to_string(), obj);
      Ok(())
    }
  }

  pub fn get_variable(&self, name: &str) -> Option<Option<Object>> {
    for scope in self.scopes.iter().rev() {
      if let Some(slot) = scope.get(name) {
        if slot.is_some() {
          return Some(slot.clone());
        } else {
          return Some(None);
        }
      }
    }
    None
  }

  pub fn assign_value(&mut self, name: &str, value: Object) -> Result<(), String> {
    for scope in self.scopes.iter_mut().rev() {
      if let Some(slot) = scope.get_mut(name) {
        if let Some(source) = slot {
          Self::reassign(source, value)?;
        } else {
          *slot = Some(value)
        }
        return Ok(());
      }
    }
    Err(format!("Undefined variable '{}'", name))
  }

  pub fn in_scope(&mut self) {
    self.scopes.push(HashMap::new());
  }

  pub fn out_scope(&mut self) {
    self.scopes.pop();
    debug_assert!(!self.scopes.is_empty(), "popped global scope")
  }

  fn reassign(source: &mut Object, target: Object) -> Result<(), String> {
    use std::mem::discriminant;
    let same_type = discriminant(source) == discriminant(&target);
    let either_nil = matches!(source, Object::Nil) || matches!(target, Object::Nil);
    if same_type || either_nil {
      *source = target;
      Ok(())
    } else {
      Err(format!("Cannot assign '{}' to '{}'", target.get_type_name(), source.get_type_name()))
    }
  }
}

impl Interpreter {
  pub fn new() -> Interpreter {
    Interpreter {
      environment: Environment::new(),
    }
  }

  pub fn interpret<'src>(&mut self, stmts: &Vec<Stmt<'src>>) -> Result<(), RuntimeError<'src>> {
    for stmt in stmts {
      self.execute_statement(stmt)?;
    }
    Ok(())
  }

  fn execute_statement<'src>(&mut self, stmt: &Stmt<'src>) -> Result<(), RuntimeError<'src>> {
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
        let value = match initializer {
          Some(expr) => Some(self.evaluate(expr)?),
          None => None,
        };
        if let Err(message) = self.environment.define_variable(name.lexeme(), value) {
          Err(RuntimeError {
            message,
            token: name.clone(),
          })
        } else {
          Ok(())
        }
      }
      Stmt::Block(stmts) => self.execute_block(stmts),
      Stmt::If { keyword, condition, then_branch, else_branch } => {
        let condition = self.evaluate(condition)?;
        if Self::object_equal(&condition, &Object::Boolean(true), &keyword)? {
          self.execute_block(then_branch)
        } else if let Some(else_branch) = else_branch {
            self.execute_block(else_branch)
        } else {
          Ok(())
        }
      }
    }
  }

  fn execute_block<'src>(&mut self, stmts: &Vec<Stmt<'src>>) -> Result<(), RuntimeError<'src>> {
    self.environment.in_scope();
    let res = stmts.iter().try_for_each(|s| self.execute_statement(s));
    self.environment.out_scope();
    res
  }

  fn evaluate<'src>(&mut self, expr: &Expr<'src>) -> Result<Object, RuntimeError<'src>> {
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
        if let Some(var) = self.environment.get_variable(name.lexeme()) {
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
      Expr::Assign { name, value } => {
        let value = self.evaluate(value)?;
        if let Err(error_message) = self.environment.assign_value(name.lexeme(), value.clone()) {
          Err(RuntimeError {
            message: error_message,
            token: name.clone(),
          })
        } else {
          Ok(value)
        }
      }
    }
  }

  fn apply_unary<'src>(
    operator: &Token<'src>,
    right_value: Object,
  ) -> Result<Object, RuntimeError<'src>> {
    match operator.ttype() {
      TokenType::Bang => Ok(Self::object_bang(&right_value, operator)?),
      TokenType::Minus => Self::object_minus(right_value, operator),
      _ => Err(RuntimeError {
        message: "Never run this line".to_string(),
        token: operator.clone(),
      }),
    }
  }

  

  fn apply_binary<'src>(
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
      TokenType::Equal => Ok(Object::Boolean(Self::object_equal(&left_value, &right_value, operator)?)),
      TokenType::BangEqual => {
        let res = Self::object_equal(
        &left_value,
        &right_value,
        operator
      )?;
      Ok(Object::Boolean(!res))
      }
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

  fn object_plus<'src>(
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

  fn object_minus<'src>(
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

  fn object_bang<'src>(right_value: &Object, token: &Token<'src>) -> Result<Object, RuntimeError<'src>> {
    match right_value {
      Object::Nil => Ok(Object::Boolean(true)),
      Object::Boolean(flag) => Ok(Object::Boolean(!flag)),
      _ => Err(RuntimeError { message: format!("Cannot reverse type '{}'", right_value.get_type_name()) , token: token.clone() })
    }
  }

  fn object_multiply<'src>(
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

  fn object_compare<'src>(
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

  fn object_equal<'src>(left: &Object, right: &Object, token: &Token<'src>) -> Result<bool, RuntimeError<'src>> {
    match (left, right) {
      (Object::Number(l), Object::Number(r)) => Ok(l == r),
      (Object::Boolean(l), Object::Boolean(r)) => Ok(l == r),
      (Object::String(l), Object::String(r)) => Ok(l == r),
      (Object::Nil, Object::Nil) => Ok(true),
      (Object::Nil, _) | (_, Object::Nil) => Ok(false),
      _ => Err(RuntimeError { message: format!("Cannot compare '{}' with '{}'", left.get_type_name(), right.get_type_name()), token: token.clone() })
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
