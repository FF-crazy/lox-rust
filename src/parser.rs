use crate::{
  SyntaxError,
  error_handling::ErrorMessage,
  expr::Expr,
  object::Object,
  token::{Token, TokenType},
};

pub struct Parser<'src> {
  tokens: Vec<Token<'src>>,
  current: usize,
}

const EQUALITY: &[TokenType] = &[TokenType::BangEqual, TokenType::Equal];
const COMPARISON: &[TokenType] = &[
  TokenType::Greater,
  TokenType::GreaterEqual,
  TokenType::Less,
  TokenType::LessEqual,
];
const FACTOR: &[TokenType] = &[TokenType::Star, TokenType::Slash];
const TERM: &[TokenType] = &[TokenType::Plus, TokenType::Minus];
// shall I add Plus to Unary? It might be useless but could be beautiful.
const UNARY: &[TokenType] = &[TokenType::Bang, TokenType::Minus];
const NUMBER_STRING: &[TokenType] = &[TokenType::String, TokenType::Number];

impl<'src> Parser<'src> {
  pub fn new(tokens: Vec<Token<'src>>) -> Self {
    Parser { tokens, current: 0 }
  }

  pub fn parse(mut self) -> Result<Expr<'src>, SyntaxError> {
    self.expression()
  }

  fn expression(&mut self) -> Result<Expr<'src>, SyntaxError> {
    self.equality()
  }

  fn equality(&mut self) -> Result<Expr<'src>, SyntaxError> {
    let mut expr = self.comparison()?;

    while let Some(operator) = self.match_one_of(EQUALITY) {
      let right = self.comparison()?;
      expr = Expr::Binary {
        left: Box::new(expr),
        operator,
        right: Box::new(right),
      };
    }
    Ok(expr)
  }

  fn comparison(&mut self) -> Result<Expr<'src>, SyntaxError> {
    let mut expr = self.term()?;
    while let Some(operator) = self.match_one_of(COMPARISON) {
      let right = self.term()?;
      expr = Expr::Binary {
        left: Box::new(expr),
        operator,
        right: Box::new(right),
      }
    }
    Ok(expr)
  }

  fn term(&mut self) -> Result<Expr<'src>, SyntaxError> {
    let mut expr = self.factor()?;
    while let Some(operator) = self.match_one_of(TERM) {
      let right = self.factor()?;
      expr = Expr::Binary {
        left: Box::new(expr),
        operator,
        right: Box::new(right),
      }
    }
    Ok(expr)
  }

  fn factor(&mut self) -> Result<Expr<'src>, SyntaxError> {
    let mut expr = self.unary()?;
    while let Some(operator) = self.match_one_of(FACTOR) {
      let right = self.unary()?;
      expr = Expr::Binary {
        left: Box::new(expr),
        operator,
        right: Box::new(right),
      }
    }
    Ok(expr)
  }

  fn unary(&mut self) -> Result<Expr<'src>, SyntaxError> {
    if let Some(operator) = self.match_one_of(UNARY) {
      let right = self.unary()?;
      Ok(Expr::Unary {
        operator,
        right: Box::new(right),
      })
    } else {
      self.primary()
    }
  }

  fn primary(&mut self) -> Result<Expr<'src>, SyntaxError> {
    // Literal
    if self.match_one_of(&[TokenType::False]).is_some() {
      return Ok(Expr::Literal(Object::Boolean(false)));
    }
    if self.match_one_of(&[TokenType::True]).is_some() {
      return Ok(Expr::Literal(Object::Boolean(true)));
    }
    if self.match_one_of(&[TokenType::Nil]).is_some() {
      return Ok(Expr::Literal(Object::Nil));
    }
    // Number or String
    if let Some(token) = self.match_one_of(NUMBER_STRING) {
      let value = token.literal().expect("It must have literal value");
      return Ok(Expr::Literal(value));
    }
    if self.match_one_of(&[TokenType::LeftParen]).is_some() {
      let expr = self.expression()?;
      self.consume(TokenType::RightParen, "expected ')' after expression")?;
      return Ok(Expr::Grouping(Box::new(expr)));
    }
    Err(SyntaxError::new(
      ErrorMessage::ExpectedExpression,
      self.current_line(),
    ))
  }

  fn consume(&mut self, ttype: TokenType, message: &str) -> Result<Token<'src>, SyntaxError> {
    if let Some(cur_token) = self.peek() {
      if cur_token.ttype() == ttype {
        return Ok(self.advance().expect("it is cur_token"));
      }
    }
    Err(SyntaxError::at(
      ErrorMessage::ExpectedToken(ttype),
      self.current_line(),
      message,
    ))
  }

  fn current_line(&self) -> usize {
    self
      .peek()
      .or_else(|| self.tokens.last())
      .map(|t| t.line())
      .unwrap_or(1)
  }

  fn match_one_of(&mut self, types: &[TokenType]) -> Option<Token<'src>> {
    let cur_type = self.peek()?.ttype();
    if types.contains(&cur_type) {
      self.advance()
    } else {
      None
    }
  }

  fn advance(&mut self) -> Option<Token<'src>> {
    let current_token = self.tokens.get(self.current).cloned();
    self.current += 1;
    current_token
  }

  fn peek(&self) -> Option<&Token<'src>> {
    self.tokens.get(self.current)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::scanner::Scanner;

  fn parse(src: &str) -> String {
    let tokens = Scanner::new(src).scan_tokens().unwrap();
    let expr = Parser::new(tokens).parse().unwrap();
    format!("{}", expr)
  }

  #[test]
  fn precedence() {
    assert_eq!(parse("1 + 2 * 3"), "(+ 1 (* 2 3))");
    assert_eq!(parse("(1 + 2) * 3"), "(* (group (+ 1 2)) 3)");
    assert_eq!(parse("-1 + 2"), "(+ (- 1) 2)");
    assert_eq!(parse("1 == 2 == 3"), "(== (== 1 2) 3)");
  }

  #[test]
  fn errors() {
    let tokens = Scanner::new("1 +").scan_tokens().unwrap();
    assert!(Parser::new(tokens).parse().is_err());

    let tokens = Scanner::new("(1 + 2").scan_tokens().unwrap();
    assert!(Parser::new(tokens).parse().is_err());
  }
}
