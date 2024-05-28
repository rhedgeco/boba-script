use crate::{
    cache::CacheSpan,
    parser::{Lexer, PError, PResult, Token},
};

use super::{Expr, Func, Node, While};

#[derive(Debug, Clone)]
pub enum Statement<Data> {
    Assign(Node<Data, String>, Node<Data, Expr<Data>>),
    LetAssign(Node<Data, String>, Node<Data, Expr<Data>>),
    Expr(Node<Data, Expr<Data>>),
    Func(Node<Data, Func<Data>>),
    While(Node<Data, While<Data>>),
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
            (Token::Ident(ident), span) => {
                let ident = Node::new(span.clone(), ident.to_string());
                tokens.next(); // consume ident

                match tokens.peek() {
                    Some(Err(error)) => Err(error),
                    None => Ok(Node::new(
                        ident.data().clone(),
                        Self::Expr(Node::new(ident.data().clone(), Expr::Var(ident))),
                    )),
                    Some(Ok((Token::Assign, _))) => {
                        tokens.next(); // consume assign
                        let rhs = Expr::parse(tokens)?;
                        tokens.expect_line_end()?;
                        let range = ident.data().range().start..rhs.data().range().end;
                        Ok(Node::new(tokens.span(range), Self::Assign(ident, rhs)))
                    }
                    Some(Ok(_)) => {
                        let lhs = Expr::parse_ident(ident, tokens)?;
                        let expr = Expr::parse_with_lhs(lhs, tokens)?;
                        tokens.expect_line_end()?;
                        Ok(Node::new(expr.data().clone(), Self::Expr(expr)))
                    }
                }
            }
            (Token::Fn, _) => {
                let func = Func::parse(tokens)?;
                Ok(Node::new(func.data().clone(), Self::Func(func)))
            }
            (Token::While, _) => {
                let r#while = While::parse(tokens)?;
                Ok(Node::new(r#while.data().clone(), Self::While(r#while)))
            }
            _ => {
                let expr = Expr::parse(tokens)?;
                tokens.expect_line_end()?;
                Ok(Node::new(expr.data().clone(), Self::Expr(expr)))
            }
        }
    }
}
