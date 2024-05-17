use crate::{
    parser::{PError, TokenSource},
    token::Span,
    Token,
};

use super::{Expr, Ident, Spanned};

#[derive(Debug)]
pub struct Assign {
    start: usize,
    ident: Ident,
    expr: Expr,
}

impl Spanned for Assign {
    fn span(&self) -> Span {
        self.start..self.expr.span().end
    }
}

impl Assign {
    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn expr(&self) -> &Expr {
        &self.expr
    }

    pub fn parse<'a>(source: &mut TokenSource) -> Result<Self, PError> {
        // match let
        let start = match source.take() {
            Some((Token::Let, span)) => span.start,
            Some((token, span)) => {
                return Err(PError::UnexpectedToken {
                    expect: format!("'{}'", Token::Let),
                    found: format!("'{token}'"),
                    span: span.clone(),
                }
                .into())
            }
            None => {
                return Err(PError::UnexpectedEnd {
                    expect: format!("'{}'", Token::Let),
                    pos: source.pos(),
                }
                .into())
            }
        };

        // parse ident
        let ident = Ident::parse(source)?;

        // match assign
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

        // build assignment
        Ok(Self { start, ident, expr })
    }
}
