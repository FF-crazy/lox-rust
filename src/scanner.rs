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
      '!' => {
        let ttype = if self.match_next('=') {TokenType::BangEqual} else {TokenType::Bang};
        self.add_token(ttype, None);
      },
      '=' => {
        let ttype = if self.match_next('=') {TokenType::Equal} else {TokenType::Assign};
        self.add_token(ttype, None);
      },
      '<' => {
        let ttype = if self.match_next('=') {TokenType::LessEqual} else {TokenType::Less};
        self.add_token(ttype, None);
      },
      '>' => {
        let ttype = if self.match_next('=') {TokenType::GreaterEqual} else {TokenType::Greater};
        self.add_token(ttype, None);
      }
      '/' => {
        if self.match_next('/') { // handling comment
          while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
          }
        }
      },
      ' ' => {},
      '\r' => {},
      '\t' => {},
      '\n' => {
        self.line += 1;
      }
      '"' => { // handling string
        while self.peek() != '=' && !self.is_at_end() {
          if self.peek() == '\n' {
            self.line += 1;
          }
          self.advance();
          if self.is_at_end() {
            return Err(LoxError::with_error(ErrorMessage::UnterminatedString, self.line, String::new()))
          }
          let value = self.source[self.start+1..self.current-1].to_string();
          self.add_token(TokenType::String, Some(Object::String(value)));
        }
      }
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
  fn match_next(&mut self, excepted: char) -> bool {
    if self.is_at_end() {
      return false
    }
    let c = self.source[self.current..].chars().next().unwrap();
    if c != excepted {
      return false
    }
    self.current += c.len_utf8();
    true
  }
  fn peek(&self) -> char {
    if self.is_at_end() {
      '\0'
    } else {
      self.source[self.current..].chars().next().unwrap()
    }
  }
}
