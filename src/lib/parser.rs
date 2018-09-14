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

    pub fn parse(&mut self) -> Option<Result<Stmt, LoxError>> {
        if self.is_at_end() {
            return None;
        }

        Some(self.declaration())
    }

    fn declaration(&mut self) -> Result<Stmt, LoxError> {
        let result = if self.is_match(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        if result.is_err() {
            self.synchronize();
        }

        result
    }

    fn var_declaration(&mut self) -> Result<Stmt, LoxError> {
        let name =
            self.consume(TokenType::Identifier, "expect variable name")?;

        let init = if self.is_match(&[TokenType::Equal]) {
            self.expression()?
        } else {
            Expr::Literal(Value::Nil)
        };

        self.consume(
            TokenType::Semicolon,
            "expect ';' after variable declaration",
        )?;

        Ok(Stmt::Var(name, init))
    }

    fn statement(&mut self) -> Result<Stmt, LoxError> {
        if self.is_match(&[TokenType::Print]) {
            self.print_statement()
        } else {
            self.expression_statement()
        }
    }

    fn print_statement(&mut self) -> Result<Stmt, LoxError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after value")?;
        Ok(Stmt::Print(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, LoxError> {
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expect ';' after expression")?;
        Ok(Stmt::Expr(value))
    }

    fn expression(&mut self) -> Result<Expr, LoxError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, LoxError> {
        let expr = self.equality()?;

        if self.is_match(&[TokenType::Equal]) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign(name, value.into()));
            }

            return Err(LoxError::parse(&equals, "Invalid assignment target."));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.comparison()?;

        while self.is_match(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let op = self.previous();
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
            let op = self.previous();
            let right = self.expression()?;
            expr = Expr::Binary(expr.into(), op, right.into());
        }

        Ok(expr)
    }

    fn addition(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.multiplication()?;

        while self.is_match(&[TokenType::Plus, TokenType::Minus]) {
            let op = self.previous();
            let right = self.expression()?;
            expr = Expr::Binary(expr.into(), op, right.into());
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.unary()?;

        while self.is_match(&[TokenType::Slash, TokenType::Star]) {
            let op = self.previous();
            let right = self.expression()?;
            expr = Expr::Binary(expr.into(), op, right.into());
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, LoxError> {
        if self.is_match(&[TokenType::Bang, TokenType::Minus]) {
            let op = self.previous();
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
            Expr::Literal(self.previous().literal)
        } else if self.is_match(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "expect ) after expression.")?;
            Expr::Grouping(expr.into())
        } else if self.is_match(&[TokenType::Identifier]) {
            Expr::Variable(self.previous())
        } else {
            return Err(LoxError::parse(self.peek(), "expect expression"));
        })
    }

    fn consume<T>(&mut self, ty: TokenType, msg: T) -> Result<Token, LoxError>
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
        if let Some(TokenType::Eof) = self.next.as_ref().map(|t| t.ty) {
            return true;
        }
        self.next.is_none()
    }

    fn advance(&mut self) -> Token {
        self.prev = self.next.take();
        self.next = self.scanner.next();
        self.previous()
    }

    fn previous(&self) -> Token {
        self.prev.as_ref().unwrap().clone()
    }
}

impl<S> Iterator for Parser<S>
where
    S: Iterator<Item = Token>,
{
    type Item = Result<Stmt, LoxError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse()
    }
}
