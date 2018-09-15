use crate::*;

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(Token, Box<Expr>),
    Binary(Box<Expr>, Token, Box<Expr>),
    Call(Box<Expr>, Token, Vec<Expr>),
    Grouping(Box<Expr>),
    Literal(Value),
    Logical(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>),
    Variable(Token),
}

pub trait Visitor<T> {
    type Output;

    fn visit(&mut self, expr: T) -> Self::Output;
}

impl<'v, V, T> Visitor<T> for &'v mut V
where
    V: Visitor<T>,
{
    type Output = <V as Visitor<T>>::Output;

    fn visit(&mut self, expr: T) -> Self::Output {
        (*self).visit(expr)
    }
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(Vec<Stmt>),
    Expr(Expr),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    Print(Expr),
    Var(Token, Expr),
    While(Expr, Box<Stmt>),
}
