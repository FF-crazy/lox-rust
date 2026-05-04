use crate::expr::Expr;

#[derive(Debug)]
pub enum Stmt<'src> {
  Expression(Expr<'src>),
  Print(Expr<'src>),
}