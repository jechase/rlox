use crate::*;

pub struct Interpreter;

impl Interpreter {
    pub fn evaluate(&mut self, expr: &Expr) -> Result<Value, LoxError> {
        self.visit(expr)
    }

    pub fn interpret(&mut self, expr: &Expr) -> Result<Value, LoxError> {
        self.evaluate(expr)
    }
}

impl Visitor<Expr> for Interpreter {
    type Output = Result<Value, LoxError>;

    fn visit(&mut self, expr: &Expr) -> Self::Output {
        Ok(match expr {
            Expr::Literal(v) => v.to_owned(),
            Expr::Grouping(e) => return self.evaluate(e),
            Expr::Unary(op, right) => {
                let right = self.evaluate(right)?;
                match op.ty {
                    TokenType::Minus => Value::Number(-*right.number()?),
                    TokenType::Bang => Value::Bool(!is_truthy(right)),
                    _ => unreachable!(),
                }
            },
            Expr::Binary(left, op, right) => {
                let left = self.evaluate(left)?;
                let right = self.evaluate(right)?;

                match op.ty {
                    TokenType::Minus => {
                        let (left, right) = number_operands(op, left, right)?;
                        Value::Number(left - right)
                    },
                    TokenType::Slash => {
                        let (left, right) = number_operands(op, left, right)?;
                        Value::Number(left / right)
                    },
                    TokenType::Star => {
                        let (left, right) = number_operands(op, left, right)?;
                        Value::Number(left * right)
                    },
                    TokenType::Plus => {
                        if let (Ok(left), Ok(right)) =
                            (left.number(), right.number())
                        {
                            Value::Number(left + right)
                        } else if let (Ok(left), Ok(right)) =
                            (left.string(), right.string())
                        {
                            let mut left = left.clone();
                            left.push_tendril(right);
                            Value::String(left)
                        } else {
                            return Err(LoxError::runtime(
                                op,
                                "requires two numbers or two strings",
                            ));
                        }
                    },
                    TokenType::Greater => {
                        let (left, right) = number_operands(op, left, right)?;
                        Value::Bool(left > right)
                    },
                    TokenType::GreaterEqual => {
                        let (left, right) = number_operands(op, left, right)?;
                        Value::Bool(left >= right)
                    },
                    TokenType::Less => {
                        let (left, right) = number_operands(op, left, right)?;
                        Value::Bool(left < right)
                    },
                    TokenType::LessEqual => {
                        let (left, right) = number_operands(op, left, right)?;
                        Value::Bool(left <= right)
                    },
                    TokenType::BangEqual => Value::Bool(!is_equal(left, right)),
                    TokenType::EqualEqual => Value::Bool(is_equal(left, right)),
                    _ => {
                        return Err(LoxError::runtime(
                            op,
                            format!("unexpected token type: {:?}", op.ty),
                        ))
                    },
                }
            },
        })
    }
}

fn is_truthy(v: Value) -> bool {
    match v {
        Value::Nil => false,
        Value::Bool(b) => b,
        _ => true,
    }
}

fn is_equal(left: Value, right: Value) -> bool {
    left == right
}

fn number_operand(op: &Token, right: Value) -> Result<f64, LoxError> {
    Ok(*right.number().map_err(|e| LoxError::runtime(op, format!("{}", e)))?)
}
fn number_operands(
    op: &Token,
    left: Value,
    right: Value,
) -> Result<(f64, f64), LoxError> {
    Ok((number_operand(op, left)?, number_operand(op, right)?))
}
