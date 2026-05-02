use crate::object::Object;
use derive_more::Display;
use std::fmt;

#[derive(Display, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
  // Single-character
  LeftParen,
  RightParen,
  LeftBrace,
  RightBrace,
  Comma,
  Dot,
  Minus,
  Plus,
  SemiColon,
  Slash,
  Star,

  // One or two character tokens
  Bang,
  BangEqual,
  Assign,
  Equal,
  Greater,
  GreaterEqual,
  Less,
  LessEqual,

  // Literals
  Identifier,
  String,
  Number,

  // Keywords
  And,
  Class,
  Else,
  False,
  Fun,
  For,
  If,
  Nil,
  Or,
  Print,
  Return,
  Super,
  This,
  True,
  Var,
  While,
  EOF,
}

#[derive(Debug, Clone)]
pub struct Token<'src> {
  ttype: TokenType,
  lexeme: &'src str,
  literal: Option<Object>,
  line: usize,
}

impl<'src> Token<'src> {
  pub fn new(ttype: TokenType, lexeme: &str, literal: Option<Object>, line: usize) -> Token<'_> {
    Token {
      ttype,
      lexeme,
      literal,
      line,
    }
  }

  pub fn lexeme(&self) -> &str {
    self.lexeme
  }

  pub fn ttype(&self) -> TokenType {
    self.ttype
  }

  pub fn literal(&self) -> Option<Object> {
    self.literal.clone()
  }

  pub fn line(&self) -> usize {
    self.line
  }
}

impl<'src> fmt::Display for Token<'src> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let literal = match &self.literal {
      Some(value) => value,
      None => &Object::Nil,
    };
    write!(f, "{} {} {}", self.ttype, self.lexeme, literal,)
  }
}
