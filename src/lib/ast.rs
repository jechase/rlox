use crate::*;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(Token, Arc<Expr>),
    Binary(Arc<Expr>, Token, Arc<Expr>),
    Call(Arc<Expr>, Token, Vec<Expr>),
    Grouping(Arc<Expr>),
    Literal(Primitive),
    Logical(Arc<Expr>, Token, Arc<Expr>),
    Unary(Token, Arc<Expr>),
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
    Function(Token, Vec<Token>, Vec<Stmt>),
    If(Expr, Arc<Stmt>, Option<Arc<Stmt>>),
    Print(Expr),
    Return(Token, Expr),
    Var(Token, Expr),
    While(Expr, Arc<Stmt>),
}
