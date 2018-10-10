use crate::*;

use display_derive::Display;

#[derive(Debug, Clone, PartialEq, Display)]
pub enum Value {
    #[display(fmt = "nil")]
    Nil,
    #[display(fmt = "{:?}", _0)]
    String(LoxStr),
    #[display(fmt = "{:?}", _0)]
    Number(f64),
    #[display(fmt = "{:?}", _0)]
    Bool(bool),
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
}
