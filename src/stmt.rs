
use crate::{expr::Expr, token::Token};

#[derive(Debug)]
pub enum Stmt<'src> {
  Expression(Expr<'src>),
  Print(Expr<'src>),
  Var {
    name: Token<'src>,
    initializer: Option<Expr<'src>>,
  },
  Block(Vec<Stmt<'src>>),
  If {
    keyword: Token<'src>,
    condition: Expr<'src>,
    then_branch: Vec<Stmt<'src>>,
    else_branch: Option<Vec<Stmt<'src>>>,
  },
  While {
    keyword: Token<'src>,
    condition: Expr<'src>,
    body: Vec<Stmt<'src>>,
  }
}
