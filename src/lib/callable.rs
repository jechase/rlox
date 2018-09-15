use std::fmt;

use crate::{
    error::LoxError,
    interpreter::Interpreter,
    value::Value,
};

pub trait Callable {
    fn call(
        &self,
        interp: &mut Interpreter,
        args: Vec<Value>,
    ) -> Result<Value, LoxError>;

    fn arity(&self) -> usize;
}

impl fmt::Debug for Callable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<callable>")
    }
}

struct LoxFn<F>(pub usize, pub F);

impl<F> Callable for LoxFn<F>
where
    F: Fn(&mut Interpreter, Vec<Value>) -> Result<Value, LoxError>,
{
    fn call(
        &self,
        interp: &mut Interpreter,
        args: Vec<Value>,
    ) -> Result<Value, LoxError> {
        (self.1)(interp, args)
    }

    fn arity(&self) -> usize {
        self.0
    }
}
