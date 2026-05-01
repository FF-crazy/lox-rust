use crate::{
  error_handling::{ErrorMessage, LoxError},
  object::Object,
  token::{Token, TokenType},
};

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
    self
      .tokens
      .push(Token::new(TokenType::EOF, "", None, self.line));
    Ok(self.tokens)
  }

  fn is_at_end(&self) -> bool {
    self.current >= self.source.len()
  }

  fn scan_token(&mut self) -> Result<(), LoxError> {
    let c = self.advance();
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
        let ttype = if self.match_next('=') {
          TokenType::BangEqual
        } else {
          TokenType::Bang
        };
        self.add_token(ttype, None);
      }
      '=' => {
        let ttype = if self.match_next('=') {
          TokenType::Equal
        } else {
          TokenType::Assign
        };
        self.add_token(ttype, None);
      }
      '<' => {
        let ttype = if self.match_next('=') {
          TokenType::LessEqual
        } else {
          TokenType::Less
        };
        self.add_token(ttype, None);
      }
      '>' => {
        let ttype = if self.match_next('=') {
          TokenType::GreaterEqual
        } else {
          TokenType::Greater
        };
        self.add_token(ttype, None);
      }
      '/' => {
        if self.match_next('/') {
          // handling comment
          self.advance_while(|c| c != '\n');
        }
      }
      ' ' => {}
      '\r' => {}
      '\t' => {}
      '\n' => {
        self.line += 1;
      }
      '"' => {
        self.handle_string()?;
      }
      '0'..='9' => {
        self.handle_digit();
      }
      other => {
        if Self::is_alpha(other) {
          self.handle_identifier();
        } else {
          return Err(LoxError::with_error(
            ErrorMessage::UnexpectedChar(other),
            self.line,
            String::new(),
          ));
        }
      }
    }
    Ok(())
  }

  fn add_token(&mut self, ttype: TokenType, literal: Option<Object>) {
    let text = &self.source[self.start..self.current];
    self
      .tokens
      .push(Token::new(ttype, text, literal, self.line));
  }

  fn advance(&mut self) -> char {
    let c = self.source[self.current..]
      .chars()
      .next()
      .expect("Only call this before checking not at end");
    self.current += c.len_utf8();
    c
  }

  fn advance_while(&mut self, func: impl Fn(char) -> bool) {
    while let Some(c) = self.peek() {
      if !func(c) {
        break;
      }
      self.advance();
    }
  }

  fn match_next(&mut self, excepted: char) -> bool {
    if let Some(c) = self.peek() {
      if c == excepted {
        self.advance();
        true
      } else {
        false
      }
    } else {
      false
    }
  }

  fn peek(&self) -> Option<char> {
    self.source[self.current..].chars().next()
  }

  fn peek_next(&self) -> Option<char> {
    let mut iter = self.source[self.current..].chars();
    iter.next()?;
    iter.next()
  }

  fn handle_string(&mut self) -> Result<(), LoxError> {
    while let Some(c) = self.peek() {
      match c {
        '"' => break,
        '\n' => self.line += 1,
        _ => {}
      }
      self.advance();
    }
    if self.is_at_end() {
      return Err(LoxError::with_error(
        ErrorMessage::UnterminatedString,
        self.line,
        String::new(),
      ));
    }
    self.advance();
    let value = self.source[self.start + 1..self.current - 1].to_string();
    self.add_token(TokenType::String, Some(Object::String(value)));
    Ok(())
  }

  fn handle_digit(&mut self) {
    self.advance_while(|c| c.is_ascii_digit());
    if self.peek() == Some('.') && self.peek_next().is_some_and(|c| c.is_ascii_digit()) {
      self.advance();
      self.advance_while(|c| c.is_ascii_digit());
    }
    let value: f64 = self.source[self.start..self.current]
      .parse()
      .expect("Never meeting parsing error");
    self.add_token(TokenType::Number, Some(Object::Number(value)));
  }

  fn handle_identifier(&mut self) {
    self.advance_while(|c| c.is_alphanumeric() || c == '_');
    if let Some(ttype) = Self::match_keyword(&self.source[self.start..self.current]) {
      self.add_token(ttype, None);
    } else {
      self.add_token(TokenType::Identifier, None);
    }
  }

  fn is_alpha(c: char) -> bool {
    c.is_alphabetic() || c == '_'
  }

  fn match_keyword(text: &str) -> Option<TokenType> {
    let ttype = match text {
      "and" => TokenType::And,
      "class" => TokenType::Class,
      "else" => TokenType::Else,
      "false" => TokenType::False,
      "for" => TokenType::For,
      "fun" => TokenType::Fun,
      "if" => TokenType::If,
      "nil" => TokenType::Nil,
      "or" => TokenType::Or,
      "print" => TokenType::Print,
      "return" => TokenType::Return,
      "super" => TokenType::Super,
      "this" => TokenType::This,
      "true" => TokenType::True,
      "var" => TokenType::Var,
      "while" => TokenType::While,
      _ => return None,
    };
    Some(ttype)
  }
}
