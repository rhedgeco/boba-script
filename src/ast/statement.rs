use crate::{
    parser::{PError, TokenSource},
    token::Span,
    Token,
};

use super::{Assign, Expr, Spanned};

#[derive(Debug)]
pub enum Statement {
    Expr(Expr),
    Assign(Assign),
}

impl Spanned for Statement {
    fn span(&self) -> Span {
        match self {
            Statement::Expr(expr) => expr.span(),
            Statement::Assign(assign) => assign.span(),
        }
    }
}

impl Statement {
    pub fn parse(source: &mut TokenSource) -> Result<Self, PError> {
        // parse expression or start assignment
        match source.peek() {
            // parse let var statement
            Some((Token::Let, _)) => Ok(Self::Assign(Assign::parse(source)?)),
            // parse expression if let not found
            Some((_, _)) => Ok(Self::Expr(Expr::parse(source)?)),
            None => Err(PError::UnexpectedEnd {
                expect: "statement".into(),
                pos: source.pos(),
            }),
        }
    }
}
