use crate::{expr::Expr, token::Token};

#[derive(Debug)]
pub enum Stmt<'src> {
  Expression(Expr<'src>),
  Print(Expr<'src>),
  Var {
    name: Token<'src>,
    initializer: Option<Expr<'src>>,
  },
}
