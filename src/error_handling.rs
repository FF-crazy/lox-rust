pub struct ErrorHandler {
  had_error: bool,
  error_message: Option<String>,
  line: Option<u32>,
  on_where: Option<String>,
}

impl ErrorHandler {
  pub fn new() -> ErrorHandler {
    ErrorHandler {
      had_error: false,
      error_message: None,
      line: None,
      on_where: None,
    }
  }

  pub fn with_error(error_message: String, line: u32, on_where: String) -> ErrorHandler {
    ErrorHandler {
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
      let error_message = self.error_message.as_deref().unwrap();
      eprintln!("[{}] Error {}: {}", line, on_where, error_message);
    }
  }
}
