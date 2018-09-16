use crate::*;

#[allow(dead_code)]
pub struct AstPrinter;

impl<'a> Visitor<&'a Expr> for AstPrinter {
    type Output = String;

    fn visit(&mut self, expr: &'a Expr) -> Self::Output {
        format!("{:#?}", expr)
    }
}

impl<'a> Visitor<&'a Stmt> for AstPrinter {
    type Output = String;

    fn visit(&mut self, stmt: &'a Stmt) -> String {
        format!("{:#?}", stmt)
    }
}
