use std::iter::Peekable;

use crate::token::{Ident, Token};

use super::{expr::Expr, ParserError, TokenIter, TokenParser};

#[derive(Debug)]
pub struct Assign {
    pub ident: Ident,
    pub expr: Expr,
}

impl TokenParser for Assign {
    type Output = Self;

    fn parse(tokens: &mut Peekable<impl TokenIter>) -> Result<Self::Output, ParserError> {
        // match let
        match tokens.next() {
            Some((Token::Let, _)) => (),
            _ => {
                return Err(ParserError {
                    message: format!("Reached end of input while parsing assignment"),
                    labels: vec![],
                })
            }
        }

        // match ident
        let ident = match tokens.next() {
            Some((Token::Ident(ident), _)) => ident.clone(),
            _ => {
                return Err(ParserError {
                    message: format!("Reached end of input while parsing assignment"),
                    labels: vec![],
                })
            }
        };

        // match equal
        match tokens.next() {
            Some((Token::Equal, _)) => (),
            _ => {
                return Err(ParserError {
                    message: format!("Reached end of input while parsing assignment"),
                    labels: vec![],
                })
            }
        }

        // match expression till end
        let expr = Expr::parse(tokens)?;

        // build assign statement
        Ok(Self { ident, expr })
    }
}
