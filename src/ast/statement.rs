use crate::{
    parser::{report::PError, Node, TokenSource},
    token::Span,
    Token,
};

use super::{Expr, Ident, LetVar};

#[derive(Debug)]
pub struct Assign {
    pub ident: Node<Ident>,
    pub expr: Node<Expr>,
}

#[derive(Debug)]
pub enum Statement {
    Expr(Node<Expr>),
    Assign(Node<Assign>),
    LetVar(Node<LetVar>),
}

impl Statement {
    pub fn span(&self) -> &Span {
        match self {
            Self::Expr(expr) => expr.span(),
            Self::Assign(assign) => assign.span(),
            Self::LetVar(letvar) => letvar.span(),
        }
    }

    pub fn parse<'a>(source: &mut impl TokenSource<'a>) -> Result<Self, PError> {
        // parse expression or start assignment
        match source.peek() {
            // parse let var statement
            Some((Token::Let, _)) => Ok(Self::LetVar(LetVar::parse(source)?)),
            // parse assign statements or expressions
            Some((Token::Ident(_), _)) => {
                let ident = Ident::parse(source)?;
                match source.peek() {
                    // parse assign
                    Some((Token::Assign, _)) => {
                        source.take(); // consume assign
                        let expr = Expr::parse(source)?;
                        let span = ident.span().start..expr.span().end;
                        Ok(Self::Assign(Node::build(span, Assign { ident, expr })))
                    }
                    // parse expression
                    Some((_, _)) => {
                        let lhs = Node::build(ident.span().clone(), Expr::Var(ident.into_inner()));
                        let expr = Expr::parse_with_lhs(lhs, source)?;
                        Ok(Self::Expr(expr))
                    }
                    None => Err(PError::UnexpectedEnd {
                        expect: format!("'{}'", Token::Assign),
                        pos: source.pos(),
                    }
                    .into()),
                }
            }
            // if let or ident is not found, parse expression normally
            Some((_, _)) => Ok(Self::Expr(Expr::parse(source)?)),
            None => Err(PError::UnexpectedEnd {
                expect: "statement".into(),
                pos: source.pos(),
            }),
        }
    }
}
