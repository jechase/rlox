use crate::LoxStr;

#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    String(LoxStr),
    Number(f64),
}
