use std::iter::Peekable;

use crate::{
    error::{Color, Label},
    token::{Ident, Token},
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
        // match let
        let span_start = match tokens.next() {
            Some((Token::Let, span)) => span.start,
            _ => {
                return Err(LangError::new(
                    "Reached end of input while parsing let statement",
                ))
            }
        };

        // match ident
        let ident = match tokens.next() {
            Some((Token::Ident(ident), _)) => ident.clone(),
            _ => {
                return Err(LangError::new(
                    "Reached end of input while parsing let statement",
                ))
            }
        };

        // match equal
        let equal_span = match tokens.next() {
            Some((Token::Equal, span)) => span,
            _ => {
                return Err(LangError::new(
                    "Reached end of input while parsing let statement",
                ))
            }
        };

        // check if there is still tokens
        if let None = tokens.peek() {
            return Err(
                LangError::new("Unexpected end of assignment").label(Label::new(
                    "nothing found after '=' token",
                    Color::Red,
                    equal_span,
                )),
            );
        }

        // match expression till end
        let expr = Expr::parse(tokens)?;

        // build assign statement
        let span = span_start..expr.span().end;
        Ok(Node::new(span, Self { ident, expr }))
    }
}
