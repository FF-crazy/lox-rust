use std::{
  cell::RefCell,
  fmt::{self, Display, Formatter},
  rc::Rc,
};

use derive_more::Display;

use crate::{interpreter::Environment, stmt::Stmt, token::Token};

pub struct LoxFunction<'src> {
  pub name: Token<'src>,
  pub parameters: Vec<Token<'src>>,
  pub body: Vec<Stmt<'src>>,
  pub closure: Rc<RefCell<Environment<'src>>>,
}

impl<'src> Display for LoxFunction<'src> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "<fn {}>", self.name.lexeme())
  }
}

impl<'src> fmt::Debug for LoxFunction<'src> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "<fn {}>", self.name.lexeme())
  }
}

#[derive(Clone, Copy)]
pub struct NativeFunction<'src> {
  pub name: &'static str,
  pub arity: usize,
  pub func: fn(Vec<Object<'src>>) -> Result<Object<'src>, String>,
}

impl<'src> Display for NativeFunction<'src> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    write!(f, "<native fn {}>", self.name)
  }
}

impl<'src> fmt::Debug for NativeFunction<'src> {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.debug_struct("NativeFunction")
      .field("name", &self.name)
      .field("arity", &self.arity)
      .finish_non_exhaustive()
  }
}

impl<'src> NativeFunction<'src> {
  pub fn new(name: &'static str, arity: usize, func: fn(Vec<Object<'src>>) -> Result<Object<'src>, String>) -> Self {
    Self {
      name,
      arity,
      func,
    }
  }
}

#[derive(Display, Debug, Clone)]
pub enum Object<'src> {
  Number(f64),
  String(String),
  Nil,
  Boolean(bool),
  Function(Rc<LoxFunction<'src>>),
  Native(NativeFunction<'src>),
}

impl<'src> Object<'src> {
  pub fn get_type_name(&self) -> &'static str {
    match self {
      Object::Number(_) => "Number",
      Object::String(_) => "String",
      Object::Boolean(_) => "Boolean",
      Object::Nil => "Nil",
      Object::Function(_) => "Function",
      Object::Native(_) => "NativeFunction",
    }
  }
}
