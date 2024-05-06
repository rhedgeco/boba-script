use std::iter::Peekable;

use crate::token::{Ident, Token};

use super::{expr::Expr, Node, BobaError, TokenIter, TokenParser};

#[derive(Debug)]
pub struct Assign {
    pub ident: Ident,
    pub expr: Node<Expr>,
}

impl TokenParser for Assign {
    type Output = Self;

    fn parse(tokens: &mut Peekable<impl TokenIter>) -> Result<Node<Self::Output>, BobaError> {
        // match let
        let span_start = match tokens.next() {
            Some((Token::Let, span)) => span.start,
            _ => {
                return Err(BobaError {
                    message: format!("Reached end of input while parsing assignment"),
                    labels: vec![],
                })
            }
        };

        // match ident
        let ident = match tokens.next() {
            Some((Token::Ident(ident), _)) => ident.clone(),
            _ => {
                return Err(BobaError {
                    message: format!("Reached end of input while parsing assignment"),
                    labels: vec![],
                })
            }
        };

        // match equal
        match tokens.next() {
            Some((Token::Equal, _)) => (),
            _ => {
                return Err(BobaError {
                    message: format!("Reached end of input while parsing assignment"),
                    labels: vec![],
                })
            }
        }

        // match expression till end
        let expr = Expr::parse(tokens)?;

        // build assign statement
        let span = span_start..expr.span().end;
        Ok(Node::new(span, Self { ident, expr }))
    }
}
