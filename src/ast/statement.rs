use crate::{
    parser::{report::PError, Node, TokenSource},
    token::Span,
    Token,
};

use super::{Expr, Ident};

#[derive(Debug)]
pub struct Assign {
    pub init: bool,
    pub ident: Node<Ident>,
    pub expr: Node<Expr>,
}

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

    pub fn parse<'a>(source: &mut impl TokenSource<'a>) -> Result<Self, PError> {
        // parse expression or start assignment
        match source.peek() {
            // parse init let statement
            Some((Token::Let, span)) => {
                let start = span.start;
                source.take(); // consume let
                let ident = Ident::parse(source)?;

                // match equal symbol
                match source.take() {
                    Some((Token::Assign, _)) => (),
                    Some((token, span)) => {
                        return Err(PError::UnexpectedToken {
                            expect: format!("'{}'", Token::Assign),
                            found: format!("'{token}'"),
                            span: span.clone(),
                        }
                        .into())
                    }
                    None => {
                        return Err(PError::UnexpectedEnd {
                            expect: format!("'{}'", Token::Assign),
                            pos: source.pos(),
                        }
                        .into())
                    }
                }

                let expr = Expr::parse(source)?;
                let span = start..expr.span().end;
                Ok(Self::Assign(Node::build(
                    span,
                    Assign {
                        init: true,
                        ident,
                        expr,
                    },
                )))
            }
            // parse set let statements or expressions
            Some((Token::Ident(_), _)) => {
                let ident = Ident::parse(source)?;
                match source.peek() {
                    Some((Token::Assign, _)) => {
                        source.take(); // consume assign
                        let expr = Expr::parse(source)?;
                        let span = ident.span().start..expr.span().end;
                        Ok(Self::Assign(Node::build(
                            span,
                            Assign {
                                init: true,
                                ident,
                                expr,
                            },
                        )))
                    }
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
            Some((token, span)) => Err(PError::UnexpectedToken {
                expect: format!("'{}'", Token::Assign),
                found: format!("'{token}'"),
                span: span.clone(),
            }
            .into()),
            None => Err(PError::UnexpectedEnd {
                expect: "statement".into(),
                pos: source.pos(),
            }),
        }
    }
}
