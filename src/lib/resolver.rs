use std::{
    collections::HashMap,
    sync::Arc,
};

use crate::*;

#[derive(Clone, Copy, Debug)]
enum FunctionType {
    None,
    Function,
}

#[derive(Debug)]
pub struct Resolver {
    scopes:   Vec<HashMap<String, bool>>,
    function: FunctionType,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            scopes:   vec![Default::default()],
            function: FunctionType::None,
        }
    }

    pub fn analyze(&mut self, mut stmt: Stmt) -> Result<Stmt, LoxError> {
        self.resolve(&mut stmt)?;
        Ok(stmt)
    }

    fn resolve_all(&mut self, stmts: &mut [Stmt]) -> Result<(), LoxError> {
        for stmt in stmts {
            self.resolve(stmt)?;
        }
        Ok(())
    }

    fn resolve(&mut self, stmt: &mut Stmt) -> Result<(), LoxError> {
        self.visit(stmt)
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new())
    }
    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn with_scope<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Resolver) -> T,
    {
        self.begin_scope();
        let res = f(self);
        self.end_scope();
        res
    }
    fn resolve_expr(&mut self, expr: &mut Expr) -> Result<(), LoxError> {
        self.visit(expr)
    }
    fn declare(&mut self, name: &Token) -> Result<(), LoxError> {
        self.scopes
            .last_mut()
            .map(|scope| {
                if scope.contains_key(name.lexeme.as_ref()) {
                    return Err(LoxError::parse(
                    name,
                    "variable with this name already declared in this scope",
                ));
                }
                scope.insert(name.lexeme.clone().into(), false);
                Ok(())
            })
            .transpose()
            .map(|_| ())
    }

    fn define(&mut self, name: &Token) {
        self.scopes
            .last_mut()
            .and_then(|scope| scope.get_mut(name.lexeme.as_ref()))
            .map(|entry| *entry = true);
    }

    fn resolve_local(&mut self, name: &Token, depth: &mut Option<usize>) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name.lexeme.as_ref()) {
                *depth = Some(i);
            }
        }
    }

    fn with_fn<F, T>(&mut self, function: FunctionType, f: F) -> T
    where
        F: FnOnce(&mut Resolver) -> T,
    {
        let enclosing = self.function;
        self.function = function;
        let res = f(self);
        self.function = enclosing;
        res
    }

    fn resolve_fn(
        &mut self,
        params: &[Token],
        body: &mut [Stmt],
        function: FunctionType,
    ) -> Result<(), LoxError> {
        self.with_fn(function, |resolver| {
            resolver.with_scope(|resolver| {
                for param in params {
                    resolver.declare(param)?;
                    resolver.define(param);
                }

                resolver.resolve_all(body)?;
                Ok(())
            })
        })
    }
}

impl<'a> Visitor<&'a mut Expr> for Resolver {
    type Output = Result<(), LoxError>;

    fn visit(&mut self, expr: &'a mut Expr) -> Self::Output {
        match expr {
            Expr::Assign(name, init, depth) => {
                let init = Arc::make_mut(init);
                self.resolve_expr(init)?;
                self.resolve_local(name, depth);
            },
            Expr::Binary(left, _, right) | Expr::Logical(left, _, right) => {
                self.resolve_expr(Arc::make_mut(left))?;
                self.resolve_expr(Arc::make_mut(right))?;
            },
            Expr::Call(callee, _, args) => {
                self.resolve_expr(Arc::make_mut(callee))?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
            },
            Expr::Grouping(expr) | Expr::Unary(_, expr) => {
                self.resolve_expr(Arc::make_mut(expr))?;
            },
            Expr::Literal(_) => {},
            Expr::Variable(name, depth) => {
                if !self.scopes.is_empty() && !self
                    .scopes
                    .last()
                    .and_then(|scope| scope.get(name.lexeme.as_ref()))
                    .map(|b| *b)
                    .unwrap_or(true)
                {
                    return Err(LoxError::parse(
                        &name,
                        "Cannot read local variable in its own initializer",
                    ));
                }

                self.resolve_local(&name, depth);
            },
        }
        Ok(())
    }
}

impl<'a> Visitor<&'a mut Stmt> for Resolver {
    type Output = Result<(), LoxError>;

    fn visit(&mut self, stmt: &'a mut Stmt) -> Self::Output {
        match stmt {
            Stmt::Block(ref mut stmts) => {
                self.with_scope(|resolver| resolver.resolve_all(stmts))?;
            },
            Stmt::Expr(expr) | Stmt::Print(expr) => {
                self.resolve_expr(expr)?;
            },
            Stmt::Function(name, params, body) => {
                self.declare(&name)?;
                self.define(&name);

                self.resolve_fn(params, body, FunctionType::Function)?;
            },
            Stmt::If(cond, then, otherwise) => {
                self.resolve_expr(cond)?;
                self.resolve(Arc::make_mut(then))?;
                if let Some(otherwise) = otherwise {
                    self.resolve(Arc::make_mut(otherwise))?;
                }
            },
            Stmt::Return(kw, expr) => {
                if let FunctionType::None = self.function {
                    return Err(LoxError::parse(
                        kw,
                        "cannot return from top level",
                    ));
                }
                self.resolve_expr(expr)?;
            },
            Stmt::Var(name, value) => {
                self.declare(name)?;
                self.resolve_expr(value)?;
                self.define(name);
            },
            Stmt::While(cond, body) => {
                self.resolve_expr(cond)?;
                self.resolve(Arc::make_mut(body))?;
            },
        }
        Ok(())
    }
}
