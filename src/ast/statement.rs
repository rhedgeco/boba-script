use crate::{
    parser::{
        report::{PError, PResult},
        Node, TokenSource,
    },
    token::Span,
    Token,
};

use super::{Assign, Expr};

#[derive(Debug)]
pub enum Statement {
    Expr(Node<Expr>),
    Assign(Node<Assign>),
}

impl Statement {
    pub fn span(&self) -> &Span {
        match self {
            Statement::Expr(expr) => expr.span(),
            Statement::Assign(assign) => assign.span(),
        }
    }

    pub fn parse<'a>(source: &mut impl TokenSource<'a>) -> PResult<Self> {
        match source.peek() {
            Some((Token::Let, _)) => Ok(Self::Assign(Assign::parse(source)?)),
            Some((_, _)) => Ok(Self::Expr(Expr::parse(source)?)),
            None => Err(PError::UnexpectedEnd {
                expect: "statement".into(),
                pos: source.pos(),
            }
            .into()),
        }
    }
}
