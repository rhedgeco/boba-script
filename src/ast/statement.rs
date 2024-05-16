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
        let (init, ident) = match source.peek() {
            Some((Token::Let, _)) => {
                source.take(); // consume let
                (true, Ident::parse(source)?)
            }
            Some((Token::Ident(_), _)) => (false, Ident::parse(source)?),
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
                    expect: "statement".into(),
                    pos: source.pos(),
                })
            }
        };

        // match assign symbol
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

        // parse expression until end
        let expr = Expr::parse(source)?;

        // build and return assignment
        let span = ident.span().start..expr.span().end;
        Ok(Self::Assign(Node::build(
            span,
            Assign { init, ident, expr },
        )))
    }
}
