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
