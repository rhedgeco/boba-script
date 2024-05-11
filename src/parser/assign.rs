use crate::{
    error::{Color, Label},
    lexer::{Ident, Token},
    BobaError,
};

use super::{parser::NodeBuilder, Expr, Node};

#[derive(Debug)]
pub struct Assign {
    pub ident: Ident,
    pub expr: Node<Expr>,
}

impl Assign {
    pub fn parser(builder: &mut NodeBuilder) -> Result<Self, BobaError> {
        const UNEXPECTED_TOKEN: &'static str = "Unexpected token found while parsing assignment";
        const UNEXPECTED_END: &'static str = "Unexpected end of assignment";

        // match let
        match builder.next() {
            Some((Token::Let, _)) => (),
            Some((token, span)) => {
                return Err(BobaError::new(UNEXPECTED_TOKEN).label(Label::new(
                    format!("expected 'let' found '{token}'"),
                    Color::Red,
                    span,
                )))
            }
            _ => {
                return Err(BobaError::new(UNEXPECTED_END).label(Label::new(
                    "Expected 'let', found nothing",
                    Color::Red,
                    builder.span().end..builder.span().end,
                )))
            }
        };

        // match ident
        let ident = match builder.next() {
            Some((Token::Ident(ident), _)) => ident,
            Some((token, span)) => {
                return Err(BobaError::new(UNEXPECTED_TOKEN).label(Label::new(
                    format!("expected 'identifier', found '{token}'"),
                    Color::Red,
                    span,
                )))
            }
            _ => {
                return Err(BobaError::new(UNEXPECTED_END).label(Label::new(
                    "expected 'identifier', found nothing",
                    Color::Red,
                    builder.span().end..builder.span().end,
                )))
            }
        };

        // match equal
        match builder.next() {
            Some((Token::Equal, span)) => span,
            Some((token, span)) => {
                return Err(BobaError::new(UNEXPECTED_TOKEN).label(Label::new(
                    format!("expected '=' found '{token}'"),
                    Color::Red,
                    span,
                )))
            }
            _ => {
                return Err(BobaError::new(UNEXPECTED_END).label(Label::new(
                    "expected '=', found nothing",
                    Color::Red,
                    builder.span().end..builder.span().end,
                )))
            }
        };

        // match expression till end
        let expr = builder.parse(Expr::parser)?;

        // build assign statement
        Ok(Self { ident, expr })
    }
}
