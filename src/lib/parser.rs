use crate::*;

use std::rc::Rc;

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
        let result = if self.is_match(&[TokenType::Fun]) {
            self.function("function")
        } else if self.is_match(&[TokenType::Class]) {
            self.class_declaration()
        } else if self.is_match(&[TokenType::Var]) {
            self.var_declaration()
        } else {
            self.statement()
        };

        if result.is_err() {
            self.synchronize();
        }

        result
    }

    fn class_declaration(&mut self) -> Result<Stmt, LoxError> {
        let name =
            self.consume(TokenType::Identifier, "expect class name")?.clone();
        self.consume(TokenType::LeftBrace, "expect '{{' before class body")?;

        let mut methods = vec![];
        while !self.check(&[TokenType::RightBrace]) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }

        self.consume(TokenType::RightBrace, "expect '}}' after class body")?;

        Ok(Stmt::Class(name, methods))
    }

    fn function(&mut self, kind: &str) -> Result<Stmt, LoxError> {
        let name = self
            .consume(TokenType::Identifier, format!("expect {} name", kind))?
            .clone();
        self.consume(
            TokenType::LeftParen,
            format!("expect '(' after {} name", kind),
        )?;

        let mut params = vec![];
        if !self.check(&[TokenType::RightParen]) {
            loop {
                if params.len() >= 8 {
                    return Err(LoxError::runtime(
                        self.peek(),
                        "cannot have more than 8 paramters",
                    ));
                }

                params.push(
                    self.consume(
                        TokenType::Identifier,
                        "expect parameter name",
                    )?
                    .clone(),
                );

                if !self.is_match(&[TokenType::Comma]) {
                    break;
                }
            }
        }
        self.consume(TokenType::RightParen, "expect ')' after parameters")?;

        self.consume(
            TokenType::LeftBrace,
            format!("expect '{{' before {} body", kind),
        )?;

        let body = self.block()?;

        Ok(Stmt::Function(name, params, body))
    }

    fn var_declaration(&mut self) -> Result<Stmt, LoxError> {
        let name = self
            .consume(TokenType::Identifier, "expect variable name")?
            .clone();

        let init = if self.is_match(&[TokenType::Equal]) {
            self.expression()?
        } else {
            Expr::Literal(Primitive::Nil)
        };

        self.consume(
            TokenType::Semicolon,
            "expect ';' after variable declaration",
        )?;

        Ok(Stmt::Var(name, init))
    }

    fn statement(&mut self) -> Result<Stmt, LoxError> {
        if self.is_match(&[TokenType::If]) {
            self.if_statement()
        } else if self.is_match(&[TokenType::For]) {
            self.for_statement()
        } else if self.is_match(&[TokenType::Print]) {
            self.print_statement()
        } else if self.is_match(&[TokenType::Return]) {
            self.return_statement()
        } else if self.is_match(&[TokenType::While]) {
            self.while_statement()
        } else if self.is_match(&[TokenType::LeftBrace]) {
            Ok(Stmt::Block(self.block()?))
        } else {
            self.expression_statement()
        }
    }

    fn return_statement(&mut self) -> Result<Stmt, LoxError> {
        let keyword = self.previous().clone();

        let expr = if !self.is_match(&[TokenType::Semicolon]) {
            Some(self.expression()?)
        } else {
            None
        };

        if expr.is_some() {
            self.consume(
                TokenType::Semicolon,
                "expect ';' after return value",
            )?;
        }

        Ok(Stmt::Return(keyword, expr))
    }

    fn for_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(TokenType::LeftParen, "expect '(' after 'for'")?;
        let decl = if self.is_match(&[TokenType::Semicolon]) {
            None
        } else if self.is_match(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(Stmt::Expr(self.expression()?))
        };

        let cond = if !self.check(&[TokenType::Semicolon]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "expect ';' after loop condition")?;

        let inc = if !self.check(&[TokenType::RightParen]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(TokenType::RightParen, "expect ')' after where clauses")?;

        let mut body = self.statement()?;

        if let Some(inc) = inc {
            body = Stmt::Block(vec![body, Stmt::Expr(inc)]);
        }

        if let Some(cond) = cond {
            body = Stmt::While(cond, body.into());
        } else {
            body =
                Stmt::While(Expr::Literal(Primitive::Bool(true)), body.into());
        }

        if let Some(decl) = decl {
            body = Stmt::Block(vec![decl, body]);
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(TokenType::LeftParen, "expect '(' after 'while'")?;
        let cond = self.expression()?;
        self.consume(
            TokenType::RightParen,
            "expect ')' after while condition",
        )?;
        let body = self.statement()?;
        Ok(Stmt::While(cond, body.into()))
    }

    fn if_statement(&mut self) -> Result<Stmt, LoxError> {
        self.consume(TokenType::LeftParen, "expect '(' after 'if'.")?;
        let cond = self.expression()?;
        self.consume(TokenType::RightParen, "expect '(' after 'if'.")?;

        let then = self.statement()?;

        let otherwise = if self.is_match(&[TokenType::Else]) {
            Some(self.statement()?)
        } else {
            None
        };

        Ok(Stmt::If(cond, then.into(), otherwise.map(From::from)))
    }

    fn block(&mut self) -> Result<Vec<Stmt>, LoxError> {
        let mut stmts = vec![];

        while !self.check(&[TokenType::RightBrace]) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "expect '}' after block.")?;

        Ok(stmts)
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
        let expr = self.or()?;

        if self.is_match(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            return match expr {
                Expr::Variable(name, depth) => {
                    Ok(Expr::Assign(name, value.into(), depth))
                },
                Expr::Get(object, name) => {
                    Ok(Expr::Set(object, name, value.into()))
                },
                _ => {
                    Err(LoxError::parse(&equals, "Invalid assignment target."))
                },
            };
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.and()?;

        while self.is_match(&[TokenType::Or]) {
            let op = self.previous().clone();

            let right = self.and()?;

            expr = Expr::Logical(expr.into(), op, right.into());
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.equality()?;

        while self.is_match(&[TokenType::And]) {
            let op = self.previous().clone();

            let right = self.equality()?;

            expr = Expr::Logical(expr.into(), op, right.into());
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.comparison()?;

        while self.is_match(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let op = self.previous().clone();
            let right = self.expression()?;
            expr = Expr::Binary(Rc::new(expr), op, Rc::new(right))
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
            let op = self.previous().clone();
            let right = self.expression()?;
            expr = Expr::Binary(expr.into(), op, right.into());
        }

        Ok(expr)
    }

    fn addition(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.multiplication()?;

        while self.is_match(&[TokenType::Plus, TokenType::Minus]) {
            let op = self.previous().clone();
            let right = self.expression()?;
            expr = Expr::Binary(expr.into(), op, right.into());
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.unary()?;

        while self.is_match(&[TokenType::Slash, TokenType::Star]) {
            let op = self.previous().clone();
            let right = self.expression()?;
            expr = Expr::Binary(expr.into(), op, right.into());
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, LoxError> {
        if self.is_match(&[TokenType::Bang, TokenType::Minus]) {
            let op = self.previous().clone();
            let right = self.unary()?;
            return Ok(Expr::Unary(op, right.into()));
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, LoxError> {
        let mut expr = self.primary()?;

        loop {
            if self.is_match(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.is_match(&[TokenType::Dot]) {
                let name = self.consume(
                    TokenType::Identifier,
                    "expect property name after '.'",
                )?;
                expr = Expr::Get(expr.into(), name.clone());
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, LoxError> {
        let mut args = vec![];

        if !self.check(&[TokenType::RightParen]) {
            loop {
                if args.len() >= 8 {
                    return Err(LoxError::parse(
                        self.peek(),
                        "cannot have more than 8 arguments",
                    ));
                }
                args.push(self.expression()?);
                if !self.is_match(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren =
            self.consume(TokenType::RightParen, "expect ')' after arguments")?;

        Ok(Expr::Call(callee.into(), paren.clone(), args))
    }

    fn primary(&mut self) -> Result<Expr, LoxError> {
        Ok(if self.is_match(&[TokenType::False]) {
            Expr::Literal(Primitive::Bool(false))
        } else if self.is_match(&[TokenType::True]) {
            Expr::Literal(Primitive::Bool(true))
        } else if self.is_match(&[TokenType::Nil]) {
            Expr::Literal(Primitive::Nil)
        } else if self.is_match(&[TokenType::This]) {
            Expr::This(self.previous().clone(), None)
        } else if self.is_match(&[TokenType::Number, TokenType::String]) {
            Expr::Literal(self.previous().literal.clone())
        } else if self.is_match(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "expect ) after expression.")?;
            Expr::Grouping(expr.into())
        } else if self.is_match(&[TokenType::Identifier]) {
            Expr::Variable(self.previous().clone(), None)
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
        if self.check(types) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, types: &[TokenType]) -> bool {
        if self.is_at_end() {
            return false;
        }
        let tok_type = self.peek().ty;
        types.contains(&tok_type)
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

    fn advance(&mut self) -> &Token {
        self.prev = self.next.take();
        self.next = self.scanner.next();
        self.previous()
    }

    fn previous(&self) -> &Token {
        self.prev.as_ref().unwrap()
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
