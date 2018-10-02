use crate::*;

use std::{
    borrow::Borrow,
    mem::swap,
    time,
};

pub struct Interpreter {
    pub environment: ScopeMgr,
    current:         Option<Index>,
}

impl Interpreter {
    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, LoxError> {
        self.visit(expr)
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<Option<Value>, LoxError> {
        self.visit(stmt)
    }

    pub fn assign(
        &mut self,
        name: &Token,
        value: Value,
    ) -> Result<(), LoxError> {
        let current = self.current;
        if self.environment.assign(current, &name.lexeme, value).is_none() {
            Err(LoxError::runtime(
                name,
                format!("variable {} is not defined", name.lexeme),
            ))
        } else {
            Ok(())
        }
    }

    pub fn define(&mut self, name: &Token, value: Value) {
        let current = self.current;
        self.environment.define(current, name.lexeme.clone(), value)
    }

    pub fn get_var(&mut self, name: &Token) -> Option<&Value> {
        let current = self.current;
        self.environment.get(current, &name.lexeme)
    }
}

impl<'a, 's> Visitor<&'a Expr> for Interpreter {
    type Output = Result<Value, LoxError>;

    fn visit(&mut self, expr: &'a Expr) -> Self::Output {
        Ok(match expr {
            Expr::Assign(name, value) => {
                let value = self.evaluate(&*value)?;
                self.assign(&name, value.clone())?;
                value
            },
            Expr::Binary(left, op, right) => {
                let left = self.evaluate(&*left)?;
                let right = self.evaluate(&*right)?;

                match op.ty {
                    TokenType::Minus => {
                        let (left, right) = number_operands(&op, left, right)?;
                        Primitive::Number(left - right)
                    },
                    TokenType::Slash => {
                        let (left, right) = number_operands(&op, left, right)?;
                        Primitive::Number(left / right)
                    },
                    TokenType::Star => {
                        let (left, right) = number_operands(&op, left, right)?;
                        Primitive::Number(left * right)
                    },
                    TokenType::Plus => {
                        if let (Ok(left), Ok(right)) = (
                            left.primitive().and_then(|p| p.number()),
                            right.primitive().and_then(|p| p.number()),
                        ) {
                            Primitive::Number(left + right).into()
                        } else if let (Ok(left), Ok(right)) = (
                            left.primitive().and_then(|p| p.string()),
                            right.primitive().and_then(|p| p.string()),
                        ) {
                            Primitive::String(
                                (left.to_string() + right.borrow()).into(),
                            )
                        } else {
                            return Err(LoxError::runtime(
                                &op,
                                "requires two numbers or two strings",
                            ));
                        }
                    },
                    TokenType::Greater => {
                        let (left, right) = number_operands(&op, left, right)?;
                        Primitive::Bool(left > right).into()
                    },
                    TokenType::GreaterEqual => {
                        let (left, right) = number_operands(&op, left, right)?;
                        Primitive::Bool(left >= right).into()
                    },
                    TokenType::Less => {
                        let (left, right) = number_operands(&op, left, right)?;
                        Primitive::Bool(left < right).into()
                    },
                    TokenType::LessEqual => {
                        let (left, right) = number_operands(&op, left, right)?;
                        Primitive::Bool(left <= right).into()
                    },
                    TokenType::BangEqual => {
                        Primitive::Bool(!is_equal(left, right)).into()
                    },
                    TokenType::EqualEqual => {
                        Primitive::Bool(is_equal(left, right)).into()
                    },
                    _ => {
                        return Err(LoxError::runtime(
                            &op,
                            format!("unexpected token type: {:?}", op.ty),
                        ))
                    },
                }
                .into()
            },
            Expr::Call(callee, paren, args) => {
                let callee = self.evaluate(&*callee)?;

                let args = args
                    .into_iter()
                    .map(|arg| self.evaluate(&arg))
                    .collect::<Result<Vec<_>, _>>()?;

                let function = callee.callable()?;

                let n_args = args.len();
                let arity = function.arity();
                if n_args != arity {
                    return Err(LoxError::runtime(
                        paren,
                        format!(
                            "expected {} arguments but got {}",
                            arity, n_args
                        ),
                    ));
                }

                function.call(self, args)?
            },
            Expr::Grouping(e) => return self.evaluate(e),
            Expr::Literal(v) => v.clone().into(),
            Expr::Logical(left, op, right) => {
                let left = self.evaluate(&*left)?;

                match op.ty {
                    TokenType::And if is_truthy(&left) => left.clone(),
                    TokenType::Or if is_truthy(&left) => left.clone(),
                    _ => self.evaluate(&*right)?,
                }
            },
            Expr::Unary(op, right) => {
                let right = self.evaluate(&*right)?;
                match op.ty {
                    TokenType::Minus => Primitive::Number(
                        -*right.primitive().and_then(|p| p.number())?,
                    ),
                    TokenType::Bang => Primitive::Bool(!is_truthy(&right)),
                    _ => unreachable!(),
                }
                .into()
            },
            Expr::Variable(name) => self
                .get_var(&name)
                .ok_or_else(|| {
                    LoxError::runtime(
                        &name,
                        format!("Undefined variable: {}", name.lexeme),
                    )
                })?
                .clone(),
        })
    }
}

fn is_truthy(v: &Value) -> bool {
    match v.primitive() {
        Ok(Primitive::Nil) => false,
        Ok(Primitive::Bool(b)) => *b,
        _ => true,
    }
}

fn is_equal(left: Value, right: Value) -> bool {
    left == right
}

fn number_operand(op: &Token, right: Value) -> Result<f64, LoxError> {
    Ok(*right
        .primitive()
        .and_then(|p| p.number())
        .map_err(|e| LoxError::runtime(op, format!("{}", e)))?)
}
fn number_operands(
    op: &Token,
    left: Value,
    right: Value,
) -> Result<(f64, f64), LoxError> {
    Ok((number_operand(op, left)?, number_operand(op, right)?))
}

impl<'a, 's> Visitor<&'a Stmt> for Interpreter {
    type Output = Result<Option<Value>, LoxError>;
    fn visit(&mut self, stmt: &'a Stmt) -> Self::Output {
        match stmt {
            Stmt::Block(stmts) => {
                let new_env = self.environment.create_scope(self.current);
                let res = self
                    .with_env(new_env, |interp| interp.execute_block(stmts));
                self.environment.destroy_scope(new_env);
                return res;
            },
            Stmt::Expr(expr) => {
                self.evaluate(expr)?;
            },
            Stmt::Function(name, params, body) => {
                if let Some(scope) = self.current {
                    self.environment.add_ref(scope);
                }
                self.define(
                    name,
                    Value::Callable(
                        LoxFn::new(name, &*params, &*body, self.current).into(),
                    ),
                )
            },
            Stmt::If(cond, then, otherwise) => {
                if is_truthy(&self.evaluate(cond)?) {
                    return self.execute(&*then);
                } else if let Some(otherwise) = otherwise {
                    return self.execute(&*otherwise);
                }
            },
            Stmt::Print(expr) => {
                println!("{}", self.evaluate(expr)?);
            },
            Stmt::Return(_, expr) => {
                let value = self.evaluate(expr)?;

                return Ok(Some(value));
            },
            Stmt::Var(name, expr) => {
                let value = self.evaluate(expr)?;
                self.define(&name, value);
            },
            Stmt::While(cond, body) => {
                while is_truthy(&self.evaluate(cond)?) {
                    if let Some(ret) = self.execute(&*body)? {
                        return Ok(Some(ret));
                    }
                }
            },
        }

        Ok(None)
    }
}

impl Default for Interpreter {
    fn default() -> Interpreter {
        Interpreter::new()
    }
}

impl Interpreter {
    fn new() -> Interpreter {
        let mut interp = Interpreter {
            environment: Default::default(),
            current:     None,
        };
        interp.environment.define(
            None,
            "clock",
            Value::Callable(
                RustFn::new(0, |_, _| {
                    Ok(Primitive::Number(
                        time::SystemTime::now()
                            .duration_since(time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs() as f64,
                    )
                    .into())
                })
                .into(),
            ),
        );
        interp
    }
    pub fn execute_block(
        &mut self,
        stmts: &Vec<Stmt>,
    ) -> Result<Option<Value>, LoxError> {
        for stmt in stmts.iter() {
            if let Some(ret) = self.execute(stmt)? {
                return Ok(Some(ret));
            }
        }
        Ok(None)
    }

    pub fn with_env<F, T>(&mut self, env: Index, f: F) -> T
    where
        F: FnOnce(&mut Interpreter) -> T,
    {
        let mut env = Some(env);
        swap(&mut self.current, &mut env);

        let res = f(self);

        swap(&mut self.current, &mut env);

        res
    }
}
