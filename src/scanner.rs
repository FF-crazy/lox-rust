use crate::token::{Token, TokenType};

pub struct Scanner {
  source: String,
  tokens: Vec<Token>,
  start: u32,
  current: u32,
  line: u32,
}

impl Scanner {
  fn new(source: String) -> Scanner {
    Scanner {
      source,
      tokens: Vec::new(),
      start: 0,
      current: 0,
      line: 1,
    }
  }
  fn scan_tokens(&mut self) -> &Vec<Token> {
    while !self.is_at_end() {
      self.start = self.current;
      self.scan_token();
    }
    self.tokens.push(Token::new(
      TokenType::EOF,
      String::from(""),
      None,
      self.line,
    ));
    &self.tokens
  }
  fn is_at_end(&self) -> bool {
    self.current >= self.source.len() as u32
  }
  fn scan_token(&self) {}
}
