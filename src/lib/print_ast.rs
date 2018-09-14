use crate::*;

#[allow(dead_code)]
pub struct AstPrinter;

impl<'r> Visitor<&'r Expr> for AstPrinter {
    type Output = String;

    fn visit(&mut self, expr: &Expr) -> Self::Output {
        format!("{:#?}", expr)
    }
}
