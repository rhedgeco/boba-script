use crate::parser::{PError, PResult, Token, TokenLine};

use super::{Expr, Node};

pub enum Statement {
    Assign(Node<String>, Node<Expr>),
    LetAssign(Node<String>, Node<Expr>),
    Expr(Node<Expr>),
}

impl Statement {
    pub fn parse(tokens: &mut TokenLine) -> PResult<Node<Self>> {
        match tokens.expect_peek("assignment or expression")? {
            (Token::Let, span) => {
                let start = span.start;
                tokens.next(); // consume let

                // capture lhs variable
                let lhs = match tokens.expect_next("identifier")? {
                    (Token::Ident(var), span) => Node::new(span, var.to_string()),
                    (token, span) => {
                        return Err(PError::UnexpectedToken {
                            expected: format!("identifier"),
                            found: format!("'{token}'"),
                            span,
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
                            span,
                        })
                    }
                };

                // capture rhs expression
                let rhs = Expr::parse(tokens)?;
                Ok(Node::new(start..rhs.span().end, Self::LetAssign(lhs, rhs)))
            }
            (Token::Ident(ident), span) => {
                let span = span.clone();
                let var = ident.to_string();
                tokens.next(); // consume ident

                match tokens.peek() {
                    Some(Err(error)) => Err(error),
                    None => Ok(Node::new(
                        span.clone(),
                        Self::Expr(Node::new(span, Expr::Var(var))),
                    )),
                    Some(Ok((Token::Assign, _))) => {
                        let lhs = Node::new(span, var);
                        tokens.next(); // consume assign
                        let rhs = Expr::parse(tokens)?;
                        tokens.expect_end()?;
                        Ok(Node::new(
                            lhs.span().start..rhs.span().end,
                            Self::Assign(lhs, rhs),
                        ))
                    }
                    Some(Ok(_)) => {
                        let lhs = Node::new(span, Expr::Var(var));
                        let expr = Expr::parse_with(lhs, tokens)?;
                        tokens.expect_end()?;
                        Ok(Node::new(expr.span().clone(), Self::Expr(expr)))
                    }
                }
            }
            _ => {
                let expr = Expr::parse(tokens)?;
                tokens.expect_end()?;
                Ok(Node::new(expr.span().clone(), Self::Expr(expr)))
            }
        }
    }
}
