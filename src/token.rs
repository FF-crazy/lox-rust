use crate::object::Object;
use derive_more::Display;
use std::fmt;

#[derive(Display)]
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

pub struct Token {
  ttype: TokenType,
  lexeme: String,
  literal: Option<Object>,
  line: u32,
}

impl Token {
  pub fn new(ttype: TokenType, lexeme: String, literal: Option<Object>, line: u32) -> Token {
    Token {
      ttype,
      lexeme,
      literal,
      line,
    }
  }
}

impl fmt::Display for Token {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let literal = match &self.literal {
      Some(value) => value,
      None => &Object::Nil,
    };
    write!(
      f,
      "{} {} {}",
      self.ttype, self.lexeme, literal,
    )
  }
}
