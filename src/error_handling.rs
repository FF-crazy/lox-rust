use std::fmt;

#[derive(Debug)]
pub struct LoxError {
  had_error: bool,
  error_message: Option<ErrorMessage>,
  line: Option<usize>,
  on_where: Option<String>,
}

impl LoxError {
  pub fn new() -> LoxError {
    LoxError {
      had_error: false,
      error_message: None,
      line: None,
      on_where: None,
    }
  }

  pub fn with_error(error_message: ErrorMessage, line: usize, on_where: String) -> LoxError {
    LoxError {
      had_error: true,
      error_message: Some(error_message),
      line: Some(line),
      on_where: Some(on_where),
    }
  }

  pub fn report(&self) {
    if self.had_error {
      let line = self.line.unwrap();
      let on_where = self.on_where.as_deref().unwrap();
      let error_message = self.error_message.as_ref().unwrap();
      eprintln!("[{}] Error {}: {}", line, on_where, error_message);
    }
  }
}

#[derive(Debug)]
pub enum ErrorMessage {
  UnexpectedChar(char),
  UnterminatedString,
}

impl fmt::Display for ErrorMessage {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ErrorMessage::UnexpectedChar(c) => write!(f, "unexpected character '{}'", c),
      ErrorMessage::UnterminatedString => write!(f, "unterminated string"),
    }
  }
}
