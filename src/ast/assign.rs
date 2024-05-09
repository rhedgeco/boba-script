use std::iter::Peekable;

use crate::{
    error::{Color, Label},
    lexer::{Ident, Token},
    LangError,
};

use super::{expr::Expr, Node, TokenIter, TokenParser};

#[derive(Debug)]
pub struct Assign {
    pub ident: Ident,
    pub expr: Node<Expr>,
}

impl TokenParser for Assign {
    type Output = Self;

    fn parse(tokens: &mut Peekable<impl TokenIter>) -> Result<Node<Self::Output>, LangError> {
        const UNEXPECTED: &'static str = "Unexpected token found while parsing assignment";
        const END_OF_INPUT: &'static str = "Unexpected end of input while parsing assignment";

        // match let
        let let_span = match tokens.next() {
            Some((Token::Let, span)) => span,
            Some((token, span)) => {
                return Err(LangError::new(UNEXPECTED).label(Label::new(
                    format!("expected 'let' found '{token}'"),
                    Color::Red,
                    span,
                )))
            }
            _ => return Err(LangError::new(END_OF_INPUT)),
        };

        // match ident
        let (ident, ident_span) = match tokens.next() {
            Some((Token::Ident(ident), span)) => (ident.clone(), span),
            Some((token, span)) => {
                return Err(LangError::new(UNEXPECTED).label(Label::new(
                    format!("expected identifier, found '{token}'"),
                    Color::Red,
                    span,
                )))
            }
            _ => {
                return Err(LangError::new(END_OF_INPUT).label(Label::new(
                    "expected identifier after 'let' but found nothing",
                    Color::Red,
                    let_span,
                )))
            }
        };

        // match equal
        let equal_span = match tokens.next() {
            Some((Token::Equal, span)) => span,
            Some((token, span)) => {
                return Err(LangError::new(UNEXPECTED).label(Label::new(
                    format!("expected '=' found '{token}'"),
                    Color::Red,
                    span,
                )))
            }
            _ => {
                return Err(LangError::new(END_OF_INPUT).label(Label::new(
                    "expected '=' after identifier but found nothing",
                    Color::Red,
                    ident_span,
                )))
            }
        };

        // check if there is still tokens
        if let None = tokens.peek() {
            return Err(LangError::new(END_OF_INPUT).label(Label::new(
                "expected expression after '=' but found nothing",
                Color::Red,
                equal_span,
            )));
        }

        // match expression till end
        let expr = Expr::parse(tokens)?;

        // build assign statement
        let span = let_span.start..expr.span().end;
        Ok(Node::new(span, Self { ident, expr }))
    }
}
