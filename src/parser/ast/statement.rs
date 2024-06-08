use crate::{
    cache::CacheSpan,
    parser::{Lexer, PError, PResult, Token},
};

use super::{Expr, Node};

#[derive(Debug, Clone)]
pub enum Statement<Data> {
    LetAssign(Node<Data, String>, Node<Data, Expr<Data>>),
    Expr(Node<Data, Expr<Data>>),
}

impl Statement<CacheSpan> {
    pub fn parse(tokens: &mut Lexer) -> PResult<CacheSpan, Node<CacheSpan, Self>> {
        match tokens.expect_peek("assignment or expression")? {
            (Token::Let, span) => {
                let start = span.range().start;
                tokens.next(); // consume let

                // capture lhs variable
                let lhs = match tokens.expect_next("identifier")? {
                    (Token::Ident(var), span) => Node::new(span, var.to_string()),
                    (token, span) => {
                        return Err(PError::UnexpectedToken {
                            expected: format!("identifier"),
                            found: format!("'{token}'"),
                            data: span,
                        })
                    }
                };

                // capture equal symbol
                match tokens.expect_next("=")? {
                    (Token::Assign, _) => (),
                    (token, span) => {
                        return Err(PError::UnexpectedToken {
                            expected: format!("="),
                            found: format!("'{token}'"),
                            data: span,
                        })
                    }
                };

                // capture rhs expression
                let rhs = Expr::parse(tokens)?;
                let span = tokens.span(start..rhs.data().range().end);
                Ok(Node::new(span, Self::LetAssign(lhs, rhs)))
            }
            _ => {
                let expr = Expr::parse(tokens)?;
                tokens.expect_line_end()?;
                Ok(Node::new(expr.data().clone(), Self::Expr(expr)))
            }
        }
    }
}
