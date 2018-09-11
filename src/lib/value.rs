use crate::*;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    String(LoxStr),
    Number(f64),
    Bool(bool),
}
