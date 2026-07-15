use crate::{
  SyntaxError,
  error_handling::ErrorMessage,
  expr::Expr,
  object::Object,
  stmt::Stmt,
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

  pub fn parse(mut self) -> Result<Vec<Stmt<'src>>, SyntaxError> {
    let mut stmts = Vec::new();
    while self.peek().is_some() {
      stmts.push(self.statement()?);
    }
    Ok(stmts)
  }

  fn statement(&mut self) -> Result<Stmt<'src>, SyntaxError> {
    if self.match_one_of(&[TokenType::Print]).is_some() {
      self.print_statement()
    } else if self.match_one_of(&[TokenType::Var]).is_some() {
      self.var_declaration()
    } else if self.match_one_of(&[TokenType::LeftBrace]).is_some() {
      self.block().map(Stmt::Block)
    } else if let Some(keyword) = self.match_one_of(&[TokenType::If]) {
      self.if_statement(keyword)
    } else if let Some(keyword) = self.match_one_of(&[TokenType::While]) {
      self.while_statement(keyword)
    } else if self.match_one_of(&[TokenType::Fun]).is_some() {
      self.function()
    } else if let Some(keyword) = self.match_one_of(&[TokenType::Return]) {
      self.return_statement(keyword)
    } else {
      self.expression_statement()
    }
  }

  fn while_statement(&mut self, keyword: Token<'src>) -> Result<Stmt<'src>, SyntaxError> {
    let condition = self.expression()?;
    self.consume(TokenType::LeftBrace, "Expect '{' after while condition")?;
    let body = self.block()?;
    Ok(Stmt::While {
      keyword,
      condition,
      body,
    })
  }

  fn if_statement(&mut self, keyword: Token<'src>) -> Result<Stmt<'src>, SyntaxError> {
    let condition = self.expression()?;
    self.consume(TokenType::LeftBrace, "Expect '{' after if condition")?;
    let then_branch = self.block()?;
    let else_branch = if self.match_one_of(&[TokenType::Else]).is_some() {
      if let Some(keyword) = self.match_one_of(&[TokenType::If]) {
        Some(vec![self.if_statement(keyword)?])
      } else {
        self.consume(TokenType::LeftBrace, "Expect '{' after else")?;
        Some(self.block()?)
      }
    } else {
      None
    };

    Ok(Stmt::If {
      keyword,
      condition,
      then_branch,
      else_branch,
    })
  }

  fn function(&mut self) -> Result<Stmt<'src>, SyntaxError> {
    let name = self.consume(TokenType::Identifier, "Expect function name here")?;
    self.consume(TokenType::LeftParen, "Expect '(' here")?;
    let mut parameters = Vec::new();
    while let Some(cur_token) = self.peek() {
      if cur_token.ttype() == TokenType::RightParen {
        break;
      }
      let p = self.consume(TokenType::Identifier, "Expect parameter name")?;
      parameters.push(p);
      if self.match_one_of(&[TokenType::Comma]).is_none() {
        break;
      }
    }
    self.consume(TokenType::RightParen, "Expect ')' after parameters")?;
    self.consume(TokenType::LeftBrace, "Expect '{' before function body")?;
    let body = self.block()?;
    Ok(Stmt::Function {
      name,
      parameters,
      body,
    })
  }

  fn return_statement(&mut self, keyword: Token<'src>) -> Result<Stmt<'src>, SyntaxError> {
    let value = if self.peek().map(|t| t.ttype()) != Some(TokenType::SemiColon) {
      Some(self.expression()?)
    } else {
      None
    };
    self.consume(TokenType::SemiColon, "Expect ';' after return value")?;
    Ok(Stmt::Return { keyword, value })
  }

  fn block(&mut self) -> Result<Vec<Stmt<'src>>, SyntaxError> {
    let mut stmts = Vec::new();
    while let Some(inner) = self.peek() {
      if inner.ttype() == TokenType::RightBrace {
        break;
      }
      stmts.push(self.statement()?);
    }
    self.consume(TokenType::RightBrace, "Expect '}' after block")?;
    Ok(stmts)
  }

  fn expression_statement(&mut self) -> Result<Stmt<'src>, SyntaxError> {
    let expr = self.expression()?;
    self.consume(TokenType::SemiColon, "Expect ';' after expression")?;
    Ok(Stmt::Expression(expr))
  }

  fn var_declaration(&mut self) -> Result<Stmt<'src>, SyntaxError> {
    let name = self.consume(TokenType::Identifier, "Expect variable name here")?;
    if self.match_one_of(&[TokenType::Assign]).is_some() {
      let initializer = self.expression()?;
      self.consume(TokenType::SemiColon, "Expect ';' after value")?;
      Ok(Stmt::Var {
        name,
        initializer: Some(initializer),
      })
    } else {
      self.consume(TokenType::SemiColon, "Expect ';' after value")?;
      Ok(Stmt::Var {
        name,
        initializer: None,
      })
    }
  }

  fn print_statement(&mut self) -> Result<Stmt<'src>, SyntaxError> {
    let value = self.expression()?;
    self.consume(TokenType::SemiColon, "Expect ';' after value")?;
    Ok(Stmt::Print(value))
  }

  fn expression(&mut self) -> Result<Expr<'src>, SyntaxError> {
    self.assignment()
  }

  fn assignment(&mut self) -> Result<Expr<'src>, SyntaxError> {
    // 先解析左边
    let expr = self.logic_or()?;

    // 递归解析右面
    if self.match_one_of(&[TokenType::Assign]).is_some() {
      let value = self.assignment()?;
      if let Expr::Variable(name) = expr {
        Ok(Expr::Assign {
          name,
          value: Box::new(value),
        })
      } else {
        Err(SyntaxError::new(
          ErrorMessage::InvalidAssignmentTarget,
          self.current_line(),
        ))
      }
    } else {
      Ok(expr)
    }
  }

  fn logic_or(&mut self) -> Result<Expr<'src>, SyntaxError> {
    let mut expr = self.logic_and()?;
    while let Some(operator) = self.match_one_of(&[TokenType::Or]) {
      let right = self.logic_and()?;
      expr = Expr::Logical {
        left: Box::new(expr),
        operator,
        right: Box::new(right),
      };
    }
    Ok(expr)
  }

  fn logic_and(&mut self) -> Result<Expr<'src>, SyntaxError> {
    let mut expr = self.equality()?;
    while let Some(operator) = self.match_one_of(&[TokenType::And]) {
      let right = self.equality()?;
      expr = Expr::Logical {
        left: Box::new(expr),
        operator,
        right: Box::new(right),
      };
    }
    Ok(expr)
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
      self.call()
    }
  }

  fn call(&mut self) -> Result<Expr<'src>, SyntaxError> {
    let mut expr = self.primary()?;
    while self.match_one_of(&[TokenType::LeftParen]).is_some() {
      expr = self.finish_call(expr)?;
    }
    Ok(expr)
  }

  fn finish_call(&mut self, callee: Expr<'src>) -> Result<Expr<'src>, SyntaxError> {
    let mut arguments = Vec::new();
    if self.peek().map(|t| t.ttype()) != Some(TokenType::RightParen) {
      loop {
        arguments.push(self.expression()?);
        if self.match_one_of(&[TokenType::Comma]).is_none() {
          break;
        }
      }
    }
    let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments")?;
    Ok(Expr::Call {
      callee: Box::new(callee),
      paren,
      arguments,
    })
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
    // Identifier
    if let Some(identifier) = self.match_one_of(&[TokenType::Identifier]) {
      return Ok(Expr::Variable(identifier));
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
    let stmts = Parser::new(tokens).parse().unwrap();
    let mut res = String::new();
    for stmt in stmts {
      res = format!("res {:?}", stmt);
    }
    res
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
