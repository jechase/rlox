use std::{
    collections::HashMap,
    rc::Rc,
};

use crate::*;

#[derive(Clone, Copy, Debug)]
enum FunctionType {
    None,
    Function,
    Method,
    Initializer,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ClassType {
    None,
    Class,
    Subclass,
}

#[derive(Debug)]
pub struct Resolver {
    scopes:   Vec<HashMap<LoxStr, bool>>,
    function: FunctionType,
    class:    ClassType,
}

impl Resolver {
    pub fn new() -> Self {
        Resolver {
            scopes:   vec![Default::default()],
            function: FunctionType::None,
            class:    ClassType::None,
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
                if scope.contains_key(&name.lexeme) {
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
            .and_then(|scope| scope.get_mut(&name.lexeme))
            .map(|entry| *entry = true);
    }

    fn resolve_local(&mut self, name: &Token, depth: &mut Option<usize>) {
        for (i, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(&name.lexeme) {
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

    fn with_class<F, T>(&mut self, class: ClassType, f: F) -> T
    where
        F: FnOnce(&mut Resolver) -> T,
    {
        let enclosing = self.class;
        self.class = class;
        let res = f(self);
        self.class = enclosing;
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
                let init = Rc::make_mut(init);
                self.resolve_expr(init)?;
                self.resolve_local(name, depth);
            },
            Expr::Binary(left, _, right) | Expr::Logical(left, _, right) => {
                self.resolve_expr(Rc::make_mut(left))?;
                self.resolve_expr(Rc::make_mut(right))?;
            },
            Expr::Call(callee, _, args) => {
                self.resolve_expr(Rc::make_mut(callee))?;
                for arg in args {
                    self.resolve_expr(arg)?;
                }
            },
            Expr::Get(expr, _) => {
                self.resolve_expr(Rc::make_mut(expr))?;
            },
            Expr::Grouping(expr) | Expr::Unary(_, expr) => {
                self.resolve_expr(Rc::make_mut(expr))?;
            },
            Expr::Literal(_) => {},
            Expr::Set(object, _, value) => {
                self.visit(Rc::make_mut(object))?;
                self.visit(Rc::make_mut(value))?;
            },
            Expr::Super(tok, _, _) if self.class == ClassType::Class => {
                return Err(LoxError::parse(tok, "super used in a class with no superclass"))
            },
            Expr::Super(tok, _, _) if self.class == ClassType::None => {
                return Err(LoxError::parse(tok, "super used outside of a class"))
            },
            Expr::Super(tok, _, depth) => self.resolve_local(tok, depth),
            Expr::This(tok, _) if ClassType::None == self.class => {
                return Err(LoxError::parse(tok, "cannot use 'this' outside of a class"))
            },
            Expr::This(tok, depth) => self.resolve_local(tok, depth),
            Expr::Variable(name, depth) => {
                if !self.scopes.is_empty() && !self
                    .scopes
                    .last()
                    .and_then(|scope| scope.get(&name.lexeme))
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
            Stmt::Class(name, superclass, methods) => {
                let class_type = if superclass.is_some() {
                    ClassType::Subclass
                } else {
                    ClassType::Class
                };
                self.with_class(class_type, |resolver| {
                    resolver.declare(name)?;

                    if let Some(superclass) = superclass {
                        resolver.resolve_expr(superclass)?;
                    }

                    resolver.define(name);

                    let resolve_class = |resolver: &mut Resolver| {
                        resolver.with_scope(|resolver| {
                            resolver.scopes.last_mut().unwrap().insert("this".into(), true);
                            for method in methods {
                                if let Stmt::Function(name, params, body) = method {
                                    let decl = if &*name.lexeme == "init" {
                                        FunctionType::Initializer
                                    } else {
                                        FunctionType::Method
                                    };
                                    resolver.resolve_fn(params, body, decl)?;
                                }
                            }
                            Ok(())
                        })
                    };

                    if superclass.is_some() {
                        resolver.with_scope(|resolver| {
                            resolver.scopes.last_mut().unwrap().insert("super".into(), true);
                            resolve_class(resolver)
                        })
                    } else {
                        resolve_class(resolver)
                    }
                })?;
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
                self.resolve(Rc::make_mut(then))?;
                if let Some(otherwise) = otherwise {
                    self.resolve(Rc::make_mut(otherwise))?;
                }
            },
            Stmt::Return(kw, expr) => {
                match self.function {
                    FunctionType::None => {
                        return Err(LoxError::parse(kw, "cannot return from top level"));
                    },
                    FunctionType::Initializer if expr.is_some() => {
                        return Err(LoxError::parse(kw, "cannot return value from initializer"));
                    },
                    _ => {},
                }
                if let Some(value) = expr {
                    self.resolve_expr(value)?;
                }
            },
            Stmt::Var(name, value) => {
                self.declare(name)?;
                self.resolve_expr(value)?;
                self.define(name);
            },
            Stmt::While(cond, body) => {
                self.resolve_expr(cond)?;
                self.resolve(Rc::make_mut(body))?;
            },
        }
        Ok(())
    }
}
