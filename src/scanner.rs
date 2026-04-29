use crate::{object::Object, token::{Token, TokenType}, error_handling::{LoxError, ErrorMessage}};

pub struct Scanner<'src> {
  source: &'src str,
  tokens: Vec<Token<'src>>,
  start: usize,
  current: usize,
  line: usize,
}

impl<'src> Scanner<'src> {
  pub fn new(source: &str) -> Scanner<'_> {
    Scanner {
      source,
      tokens: Vec::new(),
      start: 0,
      current: 0,
      line: 1,
    }
  }
  pub fn scan_tokens(mut self) -> Result<Vec<Token<'src>>, LoxError> {
    while !self.is_at_end() {
      self.start = self.current;
      self.scan_token()?;
    }
    self.tokens.push(Token::new(
      TokenType::EOF,
      "",
      None,
      self.line,
    ));
    Ok(self.tokens)
  }
  fn is_at_end(&self) -> bool {
    self.current >= self.source.len()
  }
  fn scan_token(&mut self) -> Result<(), LoxError> {
    let c= self.advance();
    match c {
      '(' => self.add_token(TokenType::LeftParen, None),
      ')' => self.add_token(TokenType::RightParen, None),
      '{' => self.add_token(TokenType::LeftBrace, None),
      '}' => self.add_token(TokenType::RightBrace, None),
      ',' => self.add_token(TokenType::Comma, None),
      '.' => self.add_token(TokenType::Dot, None),
      '-' => self.add_token(TokenType::Minus, None),
      '+' => self.add_token(TokenType::Plus, None),
      ';' => self.add_token(TokenType::SemiColon, None),
      '*' => self.add_token(TokenType::Star, None),
      other => return Err(LoxError::with_error(ErrorMessage::UnexpectedChar(other), self.line, String::new())),
    }
    Ok(())
  }
  fn add_token(&mut self, ttype: TokenType, literal: Option<Object>) {
    let text = &self.source[self.start..self.current];
    self.tokens.push(Token::new(ttype, text, literal, self.line));
  }
  fn advance(&mut self) -> char {
    let c = self.source[self.current..].chars().next().unwrap();
    self.current += c.len_utf8();
    c
  }
}
