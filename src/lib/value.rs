#[derive(Debug, Clone)]
pub enum Value {
    Nil,
    String(String),
    Number(f64),
}
