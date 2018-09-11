use crate::*;

pub struct Parser<S>
where
    S: Iterator<Item = Token>,
{
    next:    Option<Token>,
    prev:    Option<Token>,
    scanner: S,
}

impl<S> Parser<S>
where
    S: Iterator<Item = Token>,
{
    pub fn new(mut scanner: S) -> Parser<S> {
        let next = scanner.next();
        Parser {
            next,
            prev: None,
            scanner,
        }
    }

    pub fn parse(&mut self) -> Result<Expr, LoxError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, LoxError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.comparison()?;

        while self.is_match(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let op = self.previous().to_owned();
            let right = self.expression()?;
            expr = Expr::Binary(Box::new(expr), op, Box::new(right))
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.addition()?;

        while self.is_match(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = self.previous().to_owned();
            let right = self.expression()?;
            expr = Expr::Binary(expr.into(), op, right.into());
        }

        Ok(expr)
    }

    fn addition(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.multiplication()?;

        while self.is_match(&[TokenType::Plus, TokenType::Minus]) {
            let op = self.previous().to_owned();
            let right = self.expression()?;
            expr = Expr::Binary(expr.into(), op, right.into());
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.unary()?;

        while self.is_match(&[TokenType::Slash, TokenType::Star]) {
            let op = self.previous().to_owned();
            let right = self.expression()?;
            expr = Expr::Binary(expr.into(), op, right.into());
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, LoxError> {
        if self.is_match(&[TokenType::Bang, TokenType::Minus]) {
            let op = self.previous().to_owned();
            let right = self.unary()?;
            return Ok(Expr::Unary(op, right.into()));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, LoxError> {
        Ok(if self.is_match(&[TokenType::False]) {
            Expr::Literal(Value::Bool(false))
        } else if self.is_match(&[TokenType::True]) {
            Expr::Literal(Value::Bool(true))
        } else if self.is_match(&[TokenType::Nil]) {
            Expr::Literal(Value::Nil)
        } else if self.is_match(&[TokenType::Number, TokenType::String]) {
            Expr::Literal(self.previous().literal.to_owned())
        } else if self.is_match(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "expect ) after expression.")?;
            Expr::Grouping(expr.into())
        } else {
            return Err(LoxError::parse(self.peek(), "expect expression"));
        })
    }

    fn consume<T>(&mut self, ty: TokenType, msg: T) -> Result<&Token, LoxError>
    where
        T: Into<String>,
    {
        if self.peek().ty == ty {
            return Ok(self.advance());
        }

        return Err(LoxError::parse(self.peek(), msg));
    }

    #[allow(dead_code)]
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().ty == TokenType::Semicolon {
                return;
            }

            match self.peek().ty {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => {
                    return;
                },
                _ => {},
            }

            self.advance();
        }
    }

    fn is_match(&mut self, types: &[TokenType]) -> bool {
        if self.is_at_end() {
            return false;
        }
        let tok_type = self.peek().ty;
        if types.contains(&tok_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn peek(&self) -> &Token {
        self.next.as_ref().unwrap()
    }

    fn is_at_end(&self) -> bool {
        self.next.is_none()
    }

    fn advance(&mut self) -> &Token {
        self.prev = self.next.take();
        self.next = self.scanner.next();
        self.previous()
    }

    fn previous(&self) -> &Token {
        self.prev.as_ref().unwrap()
    }
}
