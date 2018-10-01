use crate::*;

use either::Either;

use std::{
    fmt,
    rc::Rc,
};

#[derive(Debug, Clone)]
pub enum Value {
    Primitive(Primitive),
    LoxFn(Rc<LoxFn>),
    RustFn(Rc<dyn Callable>),
    Class(Rc<LoxClass>),
    Instance(LoxInstance),
}

impl From<Primitive> for Value {
    fn from(other: Primitive) -> Self {
        Value::Primitive(other)
    }
}

#[derive(Debug, Clone)]
pub enum Primitive {
    Nil,
    String(LoxStr),
    Number(f64),
    Bool(bool),
}

impl From<()> for Primitive {
    fn from(_other: ()) -> Self {
        Primitive::Nil
    }
}

impl From<LoxStr> for Primitive {
    fn from(other: LoxStr) -> Self {
        Primitive::String(other)
    }
}

impl From<bool> for Primitive {
    fn from(other: bool) -> Self {
        Primitive::Bool(other)
    }
}

impl From<f64> for Primitive {
    fn from(other: f64) -> Self {
        Primitive::Number(other)
    }
}

impl PartialEq<Value> for Value {
    fn eq(&self, right: &Value) -> bool {
        match (self, right) {
            (Value::Primitive(l), Value::Primitive(r)) => r == l,
            (Value::RustFn(l), Value::RustFn(r)) => Rc::ptr_eq(l, r),
            (Value::LoxFn(l), Value::LoxFn(r)) => Rc::ptr_eq(l, r),
            (Value::Class(l), Value::Class(r)) => Rc::ptr_eq(l, r),
            (Value::Instance(l), Value::Instance(r)) => l == r,
            _ => false,
        }
    }
}

impl PartialEq<Primitive> for Primitive {
    fn eq(&self, right: &Primitive) -> bool {
        match (self, right) {
            (Primitive::Nil, Primitive::Nil) => true,
            (Primitive::String(l), Primitive::String(r)) => l.eq(r),
            (Primitive::Number(l), Primitive::Number(r)) => l.eq(r),
            (Primitive::Bool(l), Primitive::Bool(r)) => l.eq(r),
            _ => false,
        }
    }
}

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Primitive::Nil => write!(f, "nil"),
            Primitive::String(v) => write!(f, "{}", v),
            Primitive::Number(v) => write!(f, "{}", v),
            Primitive::Bool(v) => write!(f, "{}", v),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Primitive(p) => p.fmt(f),
            Value::RustFn(c) => c.fmt(f),
            Value::LoxFn(c) => c.fmt(f),
            Value::Class(c) => c.fmt(f),
            Value::Instance(i) => i.fmt(f),
        }
    }
}

macro_rules! cast_fn {
    ($fn_name:ident, $outer:ident, $variant:ident, $ret:ty) => {
        #[allow(dead_code)]
        pub fn $fn_name(&self) -> Result<&$ret, LoxError> {
            match self {
                $outer::$variant(inner) => Ok(inner),
                _ => Err(LoxError::typecast(format!("cast failed, expecting {}, actually is {:?}", stringify!($variant), self))),
            }
        }
    }
}

impl Primitive {
    cast_fn!(number, Primitive, Number, f64);
    cast_fn!(boolean, Primitive, Bool, bool);
    cast_fn!(string, Primitive, String, LoxStr);
}

impl Value {
    cast_fn!(primitive, Value, Primitive, Primitive);

    pub fn callable<'s>(&'s self) -> Result<impl Callable + 's, LoxError> {
        Ok(match self {
            Value::RustFn(inner) => Either::Left(Either::Left(inner)),
            Value::LoxFn(inner) => Either::Left(Either::Right(inner)),
            Value::Class(inner) => Either::Right(Either::Left::<_, Rc<dyn Callable>>(inner)),
            _ => {
                return Err(LoxError::typecast(format!(
                    "cast failed, expecting class or function, actually is {:?}",
                    self
                )))
            },
        })
    }
}
