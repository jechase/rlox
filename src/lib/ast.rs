use crate::*;

use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum Expr {
    Assign(Token, Rc<Expr>, Option<usize>),
    Binary(Rc<Expr>, Token, Rc<Expr>),
    Call(Rc<Expr>, Token, Vec<Expr>),
    Get(Rc<Expr>, Token),
    Grouping(Rc<Expr>),
    Literal(Primitive),
    Logical(Rc<Expr>, Token, Rc<Expr>),
    Set(Rc<Expr>, Token, Rc<Expr>),
    This(Token, Option<usize>),
    Unary(Token, Rc<Expr>),
    Variable(Token, Option<usize>),
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
    Class(Token, Vec<Stmt>),
    Expr(Expr),
    Function(Token, Vec<Token>, Vec<Stmt>),
    If(Expr, Rc<Stmt>, Option<Rc<Stmt>>),
    Print(Expr),
    Return(Token, Option<Expr>),
    Var(Token, Expr),
    While(Expr, Rc<Stmt>),
}
