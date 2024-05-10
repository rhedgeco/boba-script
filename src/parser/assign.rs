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
        const UNEXPECTED: &'static str = "Unexpected token found while parsing assignment";
        const END_OF_INPUT: &'static str = "Unexpected end of input while parsing assignment";

        // match let
        let let_span = match builder.next() {
            Some((Token::Let, span)) => span,
            Some((token, span)) => {
                return Err(BobaError::new(UNEXPECTED).label(Label::new(
                    format!("expected 'let' found '{token}'"),
                    Color::Red,
                    span,
                )))
            }
            _ => return Err(BobaError::new(END_OF_INPUT)),
        };

        // match ident
        let (ident, ident_span) = match builder.next() {
            Some((Token::Ident(ident), span)) => (ident.clone(), span),
            Some((token, span)) => {
                return Err(BobaError::new(UNEXPECTED).label(Label::new(
                    format!("expected identifier, found '{token}'"),
                    Color::Red,
                    span,
                )))
            }
            _ => {
                return Err(BobaError::new(END_OF_INPUT).label(Label::new(
                    "expected identifier after 'let' but found nothing",
                    Color::Red,
                    let_span,
                )))
            }
        };

        // match equal
        let equal_span = match builder.next() {
            Some((Token::Equal, span)) => span,
            Some((token, span)) => {
                return Err(BobaError::new(UNEXPECTED).label(Label::new(
                    format!("expected '=' found '{token}'"),
                    Color::Red,
                    span,
                )))
            }
            _ => {
                return Err(BobaError::new(END_OF_INPUT).label(Label::new(
                    "expected '=' after identifier but found nothing",
                    Color::Red,
                    ident_span,
                )))
            }
        };

        // check if there is still tokens
        if let None = builder.peek() {
            return Err(BobaError::new(END_OF_INPUT).label(Label::new(
                "expected expression after '=' but found nothing",
                Color::Red,
                equal_span,
            )));
        }

        // match expression till end
        let expr = builder.parse(Expr::parser)?;

        // build assign statement
        Ok(Self { ident, expr })
    }
}
