use crate::*;

use std::{
    collections::HashMap,
    mem::swap,
    time,
};

pub struct Interpreter {
    pub environment: Environment,
}

impl Interpreter {
    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, LoxError> {
        self.visit(expr)
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<Option<Value>, LoxError> {
        self.visit(stmt)
    }

    pub fn assign_at(
        &mut self,
        name: &Token,
        value: Value,
        depth: Option<usize>,
    ) -> Result<(), LoxError> {
        let res = if let Some(depth) = depth {
            self.environment.ancestor(depth).and_then(|mut e| e.assign(&name.lexeme, value))
        } else {
            self.environment.assign_global(&name.lexeme, value)
        };
        if res.is_none() {
            Err(LoxError::runtime(name, format!("variable {} is not defined", name.lexeme)))
        } else {
            Ok(())
        }
    }

    pub fn define(&mut self, name: &Token, value: Value) {
        self.environment.define(name.lexeme.clone(), value)
    }

    pub fn get_var_at(&mut self, name: &Token, depth: Option<usize>) -> Option<Value> {
        if let Some(depth) = depth {
            self.environment.ancestor(depth).and_then(|e| e.get(&name.lexeme))
        } else {
            self.environment.get_global(&name.lexeme)
        }
    }
}

impl<'a, 's> Visitor<&'a Expr> for Interpreter {
    type Output = Result<Value, LoxError>;

    fn visit(&mut self, expr: &'a Expr) -> Self::Output {
        Ok(match expr {
            Expr::Assign(name, value, depth) => {
                let value = self.evaluate(&*value)?;
                self.assign_at(&name, value.clone(), *depth)?;
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
                        } else if let (Ok(mut left), Ok(right)) = (
                            left.primitive().and_then(|p| p.string()).map(Clone::clone),
                            right.primitive().and_then(|p| p.string()),
                        ) {
                            left.push_tendril(right);
                            Primitive::String(left).into()
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
                    TokenType::BangEqual => Primitive::Bool(!is_equal(left, right)).into(),
                    TokenType::EqualEqual => Primitive::Bool(is_equal(left, right)).into(),
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
                        format!("expected {} arguments but got {}", arity, n_args),
                    ));
                }

                function.call(self, args)?
            },
            Expr::Grouping(e) => return self.evaluate(e),
            Expr::Get(expr, name) => {
                let object = self.evaluate(&*expr)?;
                if let Value::Instance(instance) = object {
                    instance.get(&name.lexeme).ok_or_else(|| {
                        LoxError::runtime(name, format!("undefined field: {}", name.lexeme))
                    })?
                } else {
                    return Err(LoxError::runtime(name, "only instances have fields"));
                }
            },
            Expr::Literal(v) => v.clone().into(),
            Expr::Logical(left, op, right) => {
                let left = self.evaluate(&*left)?;

                match op.ty {
                    TokenType::And if is_truthy(&left) => left.clone(),
                    TokenType::Or if is_truthy(&left) => left.clone(),
                    _ => self.evaluate(&*right)?,
                }
            },
            Expr::Set(object, name, value) => {
                let object = self.evaluate(&*object)?;
                if let Value::Instance(instance) = object {
                    let value = self.evaluate(&*value)?;
                    instance.set(name.lexeme.clone(), value.clone());
                    value
                } else {
                    return Err(LoxError::runtime(name, "only instances have fields"));
                }
            },
            Expr::Super(kw, method, depth) => {
                let superclass = self
                    .environment
                    .get_at("super".as_bytes(), *depth)
                    .map(|v| {
                        if let Value::Class(v) = v {
                            Ok(v)
                        } else {
                            Err(LoxError::runtime(kw, "super must be a class (interpreter bug)"))
                        }
                    })
                    .ok_or_else(|| {
                        LoxError::runtime(kw, "could not find superclass (interpreter bug)")
                    })??;
                let this = self
                    .environment
                    .get_at("this".as_bytes(), depth.map(|d| d - 1))
                    .map(|v| {
                        if let Value::Instance(v) = v {
                            Ok(v)
                        } else {
                            Err(LoxError::runtime(kw, "this must be an instance (interpreter bug)"))
                        }
                    })
                    .ok_or_else(|| {
                        LoxError::runtime(kw, "could not find 'this' (interpreter bug)")
                    })??;
                superclass.find_method(&this, &method.lexeme).map(|f| Value::LoxFn(f)).ok_or_else(
                    || LoxError::runtime(method, format!("undefined method: {}", method.lexeme)),
                )?
            },
            Expr::This(this, depth) => {
                self.get_var_at(this, *depth).unwrap_or_else(|| Primitive::Nil.into())
            },
            Expr::Unary(op, right) => {
                let right = self.evaluate(&*right)?;
                match op.ty {
                    TokenType::Minus => {
                        Primitive::Number(-*right.primitive().and_then(|p| p.number())?)
                    },
                    TokenType::Bang => Primitive::Bool(!is_truthy(&right)),
                    _ => unreachable!(),
                }
                .into()
            },
            Expr::Variable(name, depth) => self
                .get_var_at(&name, *depth)
                .ok_or_else(|| {
                    LoxError::runtime(&name, format!("Undefined variable: {}", name.lexeme))
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

fn number_operands(op: &Token, left: Value, right: Value) -> Result<(f64, f64), LoxError> {
    Ok((number_operand(op, left)?, number_operand(op, right)?))
}

impl<'a, 's> Visitor<&'a Stmt> for Interpreter {
    type Output = Result<Option<Value>, LoxError>;
    fn visit(&mut self, stmt: &'a Stmt) -> Self::Output {
        match stmt {
            Stmt::Block(stmts) => {
                let new_env = Environment::with_enclosing(&self.environment);
                let res = self.with_env(new_env, |interp| interp.execute_block(stmts));
                return res;
            },
            Stmt::Class(name, superclass, body) => {
                let superclass = superclass
                    .as_ref()
                    .map(|sc| {
                        let name = if let Expr::Variable(name, _) = sc {
                            name.clone()
                        } else {
                            unreachable!()
                        };
                        let sc = self.evaluate(sc)?;
                        if let Value::Class(sc) = sc {
                            Ok(sc)
                        } else {
                            Err(LoxError::runtime(&name, "superclass must be a class"))
                        }
                    })
                    .transpose()?;
                self.environment.define(name.lexeme.clone(), Primitive::Nil.into());
                let mut class_environment = self.environment.clone();
                if let Some(superclass) = superclass.clone() {
                    class_environment = Environment::with_enclosing(&class_environment);
                    class_environment.define("super", Value::Class(superclass));
                }
                let mut methods = HashMap::new();
                for method in body {
                    if let Stmt::Function(name, params, body) = method {
                        methods.insert(
                            name.lexeme.clone(),
                            LoxFn::new(
                                name,
                                params,
                                body,
                                class_environment.clone(),
                                &*name.lexeme == "init",
                            )
                            .into(),
                        );
                    } else {
                        unreachable!()
                    }
                }
                let class = LoxClass::new(name.lexeme.clone(), superclass, methods);
                self.environment.assign(&name.lexeme, Value::Class(class.into()));
            },
            Stmt::Expr(expr) => {
                self.evaluate(expr)?;
            },
            Stmt::Function(name, params, body) => self.define(
                name,
                Value::LoxFn(
                    LoxFn::new(name, params, body, self.environment.clone(), false).into(),
                ),
            ),
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
                if let Some(expr) = expr {
                    let value = self.evaluate(expr)?;
                    return Ok(Some(value));
                } else {
                    return Ok(Some(Primitive::Nil.into()));
                }
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
            environment: Environment::new(),
        };
        interp.environment.define(
            "clock",
            Value::RustFn(
                RustFn::new(0, |_, _| {
                    Ok(Primitive::Number(
                        time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs()
                            as f64,
                    )
                    .into())
                })
                .into(),
            ),
        );
        interp
    }
    pub fn execute_block(&mut self, stmts: &Vec<Stmt>) -> Result<Option<Value>, LoxError> {
        for stmt in stmts.iter() {
            if let Some(ret) = self.execute(stmt)? {
                return Ok(Some(ret));
            }
        }
        Ok(None)
    }

    pub fn with_env<F, T>(&mut self, mut env: Environment, f: F) -> T
    where
        F: FnOnce(&mut Interpreter) -> T,
    {
        swap(&mut self.environment, &mut env);

        let res = f(self);

        swap(&mut self.environment, &mut env);

        res
    }
}
