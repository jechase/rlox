use crate::*;

use either::Either;

use std::{
    fmt::{
        self,
        Debug,
        Display,
    },
    rc::Rc,
};

pub trait Callable: Debug + Display {
    fn call(&self, interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, LoxError>;

    fn arity(&self) -> usize;
}

pub struct RustFn<F>(pub usize, pub F);

impl<F> Display for RustFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<native fn>")
    }
}

impl<F> Debug for RustFn<F> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<native fn>")
    }
}

impl<F> RustFn<F>
where
    F: Fn(&mut Interpreter, Vec<Value>) -> Result<Value, LoxError>,
{
    pub fn new(arity: usize, f: F) -> RustFn<F> {
        RustFn(arity, f)
    }
}

impl<'c, F> Callable for RustFn<F>
where
    F: Fn(&mut Interpreter, Vec<Value>) -> Result<Value, LoxError>,
{
    fn call(&self, interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, LoxError> {
        (self.1)(interp, args)
    }

    fn arity(&self) -> usize {
        self.0
    }
}

impl<'c, F> From<RustFn<F>> for Rc<dyn Callable>
where
    F: Fn(&mut Interpreter, Vec<Value>) -> Result<Value, LoxError> + 'static,
{
    fn from(other: RustFn<F>) -> Rc<dyn Callable> {
        Rc::new(other) as Rc<_>
    }
}

#[derive(Debug, Clone)]
pub struct LoxFn {
    fn_static: Rc<FnStatic>,
    closure:   Environment,
}

#[derive(Debug)]
struct FnStatic {
    name:    Token,
    params:  Vec<Token>,
    body:    Vec<Stmt>,
    is_init: bool,
}

impl Display for LoxFn {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn {}>", self.fn_static.name.lexeme)
    }
}

impl From<LoxFn> for Rc<dyn Callable> {
    fn from(other: LoxFn) -> Self {
        Rc::new(other) as _
    }
}

impl LoxFn {
    pub fn new(
        name: &Token,
        params: &[Token],
        body: &[Stmt],
        closure: Environment,
        is_init: bool,
    ) -> Self {
        LoxFn {
            fn_static: FnStatic {
                name: name.clone(),
                params: params.into(),
                body: body.into(),
                is_init,
            }
            .into(),
            closure,
        }
    }

    pub fn bind(&self, this: LoxInstance) -> LoxFn {
        let mut closure = Environment::with_enclosing(&self.closure);
        closure.define("this", Value::Instance(this));
        LoxFn {
            fn_static: self.fn_static.clone(),
            closure,
        }
    }
}

impl Callable for LoxFn {
    fn call(&self, interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, LoxError> {
        let env = Environment::with_enclosing(&self.closure);
        let res = interp.with_env(env, |interp| {
            for (i, decl_param) in self.fn_static.params.iter().enumerate() {
                interp.define(decl_param, args[i].clone());
            }

            interp.execute_block(&self.fn_static.body)
        });

        if self.fn_static.is_init {
            if let Some(this) = self.closure.get_at("this".as_bytes(), 0) {
                return Ok(this);
            }
        }

        res.map(|opt| opt.unwrap_or_else(|| Primitive::Nil.into()))
    }

    fn arity(&self) -> usize {
        self.fn_static.params.len()
    }
}

impl<T, U> Callable for Either<T, U>
where
    T: Callable,
    U: Callable,
{
    fn call(&self, interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, LoxError> {
        match self {
            Either::Right(c) => c.call(interp, args),
            Either::Left(c) => c.call(interp, args),
        }
    }

    fn arity(&self) -> usize {
        match self {
            Either::Right(c) => c.arity(),
            Either::Left(c) => c.arity(),
        }
    }
}

impl<T> Callable for &T
where
    T: Callable,
{
    fn call(&self, interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, LoxError> {
        (*self).call(interp, args)
    }

    fn arity(&self) -> usize {
        (*self).arity()
    }
}

impl Callable for Rc<dyn Callable> {
    fn call(&self, interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, LoxError> {
        (**self).call(interp, args)
    }

    fn arity(&self) -> usize {
        (**self).arity()
    }
}

impl<T> Callable for Rc<T>
where
    T: Callable,
{
    fn call(&self, interp: &mut Interpreter, args: Vec<Value>) -> Result<Value, LoxError> {
        (**self).call(interp, args)
    }

    fn arity(&self) -> usize {
        (**self).arity()
    }
}
