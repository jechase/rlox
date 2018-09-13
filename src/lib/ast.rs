use crate::*;

#[derive(Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Value),
    Unary(Token, Box<Expr>),
    Variable(Token),
}

pub trait Visitor<T> {
    type Output;

    fn visit(&mut self, expr: &T) -> Self::Output;
}

impl<'v, V, T> Visitor<T> for &'v mut V
where
    V: Visitor<T>,
{
    type Output = <V as Visitor<T>>::Output;

    fn visit(&mut self, expr: &T) -> Self::Output {
        (*self).visit(expr)
    }
}

#[derive(Debug)]
pub enum Stmt {
    Expr(Expr),
    Print(Expr),
    Var(Token, Expr),
}
