use crate::*;

use std::{
    fmt,
    rc::Rc,
};

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    String(LoxStr),
    Number(f64),
    Bool(bool),
    Callable(Rc<dyn Callable>),
}

impl PartialEq<Value> for Value {
    fn eq(&self, right: &Value) -> bool {
        match (self, right) {
            (Value::Nil, Value::Nil) => true,
            (Value::String(l), Value::String(r)) => l.eq(r),
            (Value::Number(l), Value::Number(r)) => l.eq(r),
            (Value::Bool(l), Value::Bool(r)) => l.eq(r),
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Nil => write!(f, "nil"),
            Value::String(v) => write!(f, "{}", v),
            Value::Number(v) => write!(f, "{}", v),
            Value::Bool(v) => write!(f, "{}", v),
            Value::Callable(_) => write!(f, "<function>"),
        }
    }
}

macro_rules! cast_fn {
    ($fn_name:ident, $variant:ident, $ret:ty) => {
        #[allow(dead_code)]
        pub fn $fn_name(&self) -> Result<&$ret, LoxError> {
            match self {
                Value::$variant(inner) => Ok(inner),
                _ => Err(LoxError::typecast(format!("cast failed, expecting {}, actually is {:?}", stringify!($variant), self))),
            }
        }
    }
}

impl Value {
    cast_fn!(number, Number, f64);
    cast_fn!(boolean, Bool, bool);
    cast_fn!(string, String, LoxStr);
    cast_fn!(callable, Callable, Rc<Callable>);
}
