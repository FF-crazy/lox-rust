use crate::{
  error_handling::RuntimeError,
  expr::Expr,
  object::{LoxFunction, Object},
  stmt::Stmt,
  token::{
    Token,
    TokenType::{self},
  },
};

use std::collections::HashMap;
use std::{cell::RefCell, cmp::Ordering, rc::Rc};

enum Flow<'src> {
  Normal,
  Return(Object<'src>),
}

pub struct Interpreter<'src> {
  environment: Rc<RefCell<Environment<'src>>>,
}

#[derive(Debug)]
pub struct Environment<'src> {
  values: HashMap<String, Option<Object<'src>>>,
  enclosing: Option<Rc<RefCell<Environment<'src>>>>,
}

impl<'src> Environment<'src> {
  pub fn new() -> Environment<'src> {
    Environment {
      values: HashMap::new(),
      enclosing: None,
    }
  }
  pub fn with_enclosing(parent: Rc<RefCell<Environment<'src>>>) -> Environment<'src> {
    Environment {
      values: HashMap::new(),
      enclosing: Some(parent),
    }
  }

  pub fn define_variable(&mut self, name: &str, obj: Option<Object<'src>>) -> Result<(), String> {
    if self.values.contains_key(name) {
      Err(format!(
        "Variable '{}' is already declared in this scope ",
        name
      ))
    } else {
      self.values.insert(name.to_string(), obj);
      Ok(())
    }
  }

  pub fn get_variable(&self, name: &str) -> Option<Option<Object<'src>>> {
    if let Some(slot) = self.values.get(name) {
      Some(slot.clone())
    } else if let Some(parent) = &self.enclosing {
      parent.borrow().get_variable(name)
    } else {
      None
    }
  }

  pub fn assign_value(&mut self, name: &str, value: Object<'src>) -> Result<(), String> {
    if let Some(slot) = self.values.get_mut(name) {
      if let Some(source) = slot {
        Self::reassign(source, value)?;
        Ok(())
      } else {
        *slot = Some(value);
        Ok(())
      }
    } else if let Some(parent) = &self.enclosing {
      parent.borrow_mut().assign_value(name, value)
    } else {
      Err(format!("Undefined variable '{}'", name))
    }
  }

  fn reassign(source: &mut Object<'src>, target: Object<'src>) -> Result<(), String> {
    use std::mem::discriminant;
    let same_type = discriminant(source) == discriminant(&target);
    let either_nil = matches!(source, Object::Nil) || matches!(target, Object::Nil);
    if same_type || either_nil {
      *source = target;
      Ok(())
    } else {
      Err(format!(
        "Cannot assign '{}' to '{}'",
        target.get_type_name(),
        source.get_type_name()
      ))
    }
  }
}

impl<'src> Interpreter<'src> {
  pub fn new() -> Interpreter<'src> {
    Interpreter {
      environment: Rc::new(RefCell::new(Environment::new())),
    }
  }

  pub fn interpret(&mut self, stmts: &Vec<Stmt<'src>>) -> Result<(), RuntimeError<'src>> {
    for stmt in stmts {
      self.execute_statement(stmt)?;
    }
    Ok(())
  }

  fn execute_statement(&mut self, stmt: &Stmt<'src>) -> Result<Flow<'src>, RuntimeError<'src>> {
    match stmt {
      Stmt::Expression(expr) => {
        self.evaluate(expr)?;
        Ok(Flow::Normal)
      }
      Stmt::Print(expr) => {
        let value = self.evaluate(expr)?;
        println!("{}", value);
        Ok(Flow::Normal)
      }
      Stmt::Var { name, initializer } => {
        let value = match initializer {
          Some(expr) => Some(self.evaluate(expr)?),
          None => None,
        };
        if let Err(message) = self
          .environment
          .borrow_mut()
          .define_variable(name.lexeme(), value)
        {
          Err(RuntimeError {
            message,
            token: name.clone(),
          })
        } else {
          Ok(Flow::Normal)
        }
      }
      Stmt::Block(stmts) => self.execute_block(stmts),
      Stmt::If {
        keyword,
        condition,
        then_branch,
        else_branch,
      } => {
        let condition = self.evaluate(condition)?;
        let cond = Self::require_bool_or_nil(&condition, &keyword)?;
        if cond {
          self.execute_block(then_branch)
        } else if let Some(else_branch) = else_branch {
          self.execute_block(else_branch)
        } else {
          Ok(Flow::Normal)
        }
      }
      Stmt::While {
        keyword,
        condition,
        body,
      } => {
        loop {
          let value = self.evaluate(condition)?;
          if !Self::require_bool_or_nil(&value, keyword)? {
            break;
          }
          match self.execute_block(body)? {
            Flow::Normal => {}
            Flow::Return(val) => {
              return Ok(Flow::Return(val));
            }
          }
        }
        Ok(Flow::Normal)
      }
      Stmt::Function {
        name,
        parameters,
        body,
      } => {
        let function = LoxFunction {
          name: name.clone(),
          parameters: parameters.clone(),
          body: body.clone(),
          closure: Rc::clone(&self.environment),
        };
        let value = Object::Function(Rc::new(function));
        self
          .environment
          .borrow_mut()
          .define_variable(name.lexeme(), Some(value))
          .map_err(|message| RuntimeError {
            message,
            token: name.clone(),
          })?;
          Ok(Flow::Normal)
      }
      Stmt::Return { keyword: _, value } => {
        let res = match value {
          Some(expr) => self.evaluate(expr)?,
          None => Object::Nil,
        };
        Ok(Flow::Return(res))
      }
    }
  }

  fn execute_block(&mut self, stmts: &Vec<Stmt<'src>>) -> Result<Flow<'src>, RuntimeError<'src>> {
    let parent = Rc::clone(&self.environment);
    let child = Rc::new(RefCell::new(Environment::with_enclosing(Rc::clone(
      &parent,
    ))));
    self.environment = child;
    let mut res = Ok(Flow::Normal);
    for s in stmts {
      match self.execute_statement(s) {
        Ok(Flow::Normal) => {}
        Ok(Flow::Return(val)) => {
          res = Ok(Flow::Return(val));
          break;
        }
        Err(e) => {
          res = Err(e);
          break;
        }
      }
    }
    self.environment = parent;
    res
  }

  fn evaluate(&mut self, expr: &Expr<'src>) -> Result<Object<'src>, RuntimeError<'src>> {
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
        if let Some(var) = self.environment.borrow().get_variable(name.lexeme()) {
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
        if let Err(error_message) = self
          .environment
          .borrow_mut()
          .assign_value(name.lexeme(), value.clone())
        {
          Err(RuntimeError {
            message: error_message,
            token: name.clone(),
          })
        } else {
          Ok(value)
        }
      }
      Expr::Logical {
        left,
        operator,
        right,
      } => {
        let left_val = self.evaluate(left)?;
        let left_bool = Self::require_bool_or_nil(&left_val, operator)?;
        let res = match operator.ttype() {
          TokenType::Or if left_bool => true,    // 短路
          TokenType::And if !left_bool => false, // 短路
          _ => {
            let right_val = self.evaluate(right)?;
            Self::require_bool_or_nil(&right_val, operator)?
          }
        };
        Ok(Object::Boolean(res))
      }
      Expr::Call {
        callee,
        paren,
        arguments,
      } => {
        let callee_val = self.evaluate(callee)?;
        let mut args = Vec::new();
        for arg in arguments {
          args.push(self.evaluate(arg)?);
        }
        let function = match callee_val {
          Object::Function(f) => f,
          other => {
            return Err(RuntimeError {
              message: format!("Can only call functions, got '{}'", other.get_type_name()),
              token: paren.clone(),
            });
          }
        };
        if args.len() != function.parameters.len() {
          Err(RuntimeError {
            message: format!(
              "Expected {} arguments but got {}",
              function.parameters.len(),
              args.len()
            ),
            token: paren.clone(),
          })
        } else {
          self.call_function(&function, args)
        }
      }
    }
  }

  fn call_function(
    &mut self,
    function: &LoxFunction<'src>,
    args: Vec<Object<'src>>,
  ) -> Result<Object<'src>, RuntimeError<'src>> {
    let env = Rc::new(RefCell::new(Environment::with_enclosing(Rc::clone(
      &function.closure,
    ))));
    for (param, arg) in function.parameters.iter().zip(args) {
      env
        .borrow_mut()
        .define_variable(param.lexeme(), Some(arg))
        .map_err(|message| RuntimeError {
          message,
          token: param.clone(),
        })?;
    }
    let previous = Rc::clone(&self.environment);
    self.environment = env;
    let mut res = Ok(Object::Nil);
    for s in &function.body {
      match self.execute_statement(s) {
        Ok(Flow::Normal) => {}
        Ok(Flow::Return(val)) => {
          res = Ok(val);
          break;
        }
        Err(e) => {
          res = Err(e);
          break;
        }
      }
    }
    self.environment = previous;
    res
  }

  fn apply_unary(
    operator: &Token<'src>,
    right_value: Object<'src>,
  ) -> Result<Object<'src>, RuntimeError<'src>> {
    match operator.ttype() {
      TokenType::Bang => Ok(Self::object_bang(&right_value, operator)?),
      TokenType::Minus => Self::object_minus(right_value, operator),
      _ => Err(RuntimeError {
        message: "Never run this line".to_string(),
        token: operator.clone(),
      }),
    }
  }

  fn apply_binary(
    left_value: Object<'src>,
    operator: &Token<'src>,
    right_value: Object<'src>,
  ) -> Result<Object<'src>, RuntimeError<'src>> {
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
      TokenType::Equal => Ok(Object::Boolean(Self::object_equal(
        &left_value,
        &right_value,
        operator,
      )?)),
      TokenType::BangEqual => {
        let res = Self::object_equal(&left_value, &right_value, operator)?;
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

  fn object_plus(
    left: Object<'src>,
    right: Object<'src>,
    token: &Token<'src>,
  ) -> Result<Object<'src>, RuntimeError<'src>> {
    match (left, right) {
      (Object::Number(l), Object::Number(r)) => Ok(Object::Number(l + r)),
      (Object::String(l), Object::String(r)) => Ok(Object::String(l + &r)),
      _ => Err(RuntimeError {
        message: "'+' Operand must be Number or String".to_string(),
        token: token.clone(),
      }),
    }
  }

  fn object_minus(
    right_value: Object<'src>,
    operator: &Token<'src>,
  ) -> Result<Object<'src>, RuntimeError<'src>> {
    match right_value {
      Object::Number(value) => Ok(Object::Number(-value)),
      _ => Err(RuntimeError {
        message: String::from("Operand must be a Number"),
        token: operator.clone(),
      }),
    }
  }

  fn object_bang(
    right_value: &Object<'src>,
    token: &Token<'src>,
  ) -> Result<Object<'src>, RuntimeError<'src>> {
    match right_value {
      Object::Nil => Ok(Object::Boolean(true)),
      Object::Boolean(flag) => Ok(Object::Boolean(!flag)),
      _ => Err(RuntimeError {
        message: format!("Cannot reverse type '{}'", right_value.get_type_name()),
        token: token.clone(),
      }),
    }
  }

  fn object_multiply(
    left: Object<'src>,
    right: Object<'src>,
    token: &Token<'src>,
  ) -> Result<Object<'src>, RuntimeError<'src>> {
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
    left: Object<'src>,
    right: Object<'src>,
    token: &Token<'src>,
    cmp: impl Fn(Ordering) -> bool,
  ) -> Result<Object<'src>, RuntimeError<'src>> {
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

  fn object_equal(
    left: &Object<'src>,
    right: &Object<'src>,
    token: &Token<'src>,
  ) -> Result<bool, RuntimeError<'src>> {
    match (left, right) {
      (Object::Number(l), Object::Number(r)) => Ok(l == r),
      (Object::Boolean(l), Object::Boolean(r)) => Ok(l == r),
      (Object::String(l), Object::String(r)) => Ok(l == r),
      (Object::Nil, Object::Nil) => Ok(true),
      (Object::Nil, _) | (_, Object::Nil) => Ok(false),
      _ => Err(RuntimeError {
        message: format!(
          "Cannot compare '{}' with '{}'",
          left.get_type_name(),
          right.get_type_name()
        ),
        token: token.clone(),
      }),
    }
  }

  fn require_bool_or_nil(
    value: &Object<'src>,
    token: &Token<'src>,
  ) -> Result<bool, RuntimeError<'src>> {
    match value {
      Object::Boolean(b) => Ok(*b),
      Object::Nil => Ok(false),
      other => Err(RuntimeError {
        message: format!(
          "'{}' operand must be Bool or Nil, got '{}'",
          token.lexeme(),
          other.get_type_name()
        ),
        token: token.clone(),
      }),
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
    let _ = i.interpret(&stmts);
  }

  #[test]
  fn test() {}
}
