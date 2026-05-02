use std::error::Error;
use std::fmt::{self, Display, Formatter};

use crate::token::TokenType;

#[derive(Debug)]
pub enum LoxError {
  Syntax(SyntaxError),
  Runtime(RuntimeError),
}

impl LoxError {
  pub fn report(&self) {
    eprintln!("{self}");
  }
}

impl Display for LoxError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::Syntax(error) => Display::fmt(error, f),
      Self::Runtime(error) => Display::fmt(error, f),
    }
  }
}

impl Error for LoxError {
  fn source(&self) -> Option<&(dyn Error + 'static)> {
    match self {
      Self::Syntax(error) => Some(error),
      Self::Runtime(error) => Some(error),
    }
  }
}

impl From<SyntaxError> for LoxError {
  fn from(error: SyntaxError) -> Self {
    Self::Syntax(error)
  }
}

impl From<RuntimeError> for LoxError {
  fn from(error: RuntimeError) -> Self {
    Self::Runtime(error)
  }
}

#[derive(Debug)]
pub struct SyntaxError {
  message: ErrorMessage,
  line: usize,
  location: Option<String>,
}

impl SyntaxError {
  pub fn new(message: ErrorMessage, line: usize) -> Self {
    Self {
      message,
      line,
      location: None,
    }
  }

  pub fn at(message: ErrorMessage, line: usize, location: impl Into<String>) -> Self {
    Self {
      message,
      line,
      location: Some(location.into()),
    }
  }

  pub fn report(&self) {
    eprintln!("{self}");
  }
}

impl Display for SyntaxError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self
      .location
      .as_deref()
      .filter(|location| !location.is_empty())
    {
      Some(location) => write!(
        f,
        "[line {}] Error {}: {}",
        self.line, location, self.message
      ),
      None => write!(f, "[line {}] Error: {}", self.line, self.message),
    }
  }
}

impl Error for SyntaxError {}

#[derive(Debug)]
pub struct RuntimeError {}

impl Display for RuntimeError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    f.write_str("runtime error")
  }
}

impl Error for RuntimeError {}

#[derive(Debug)]
pub enum ErrorMessage {
  UnexpectedChar(char),
  UnterminatedString,
  UnterminatedComment,
  ExpectedExpression,
  ExpectedToken(TokenType),
}

impl Display for ErrorMessage {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      Self::UnexpectedChar(c) => write!(f, "unexpected character '{c}'"),
      Self::UnterminatedString => f.write_str("unterminated string"),
      Self::UnterminatedComment => f.write_str("unterminated block comment"),
      Self::ExpectedExpression => f.write_str("Not a valid expression"),
      Self::ExpectedToken(t) => write!(f, "expected token '{}'", t),
    }
  }
}
