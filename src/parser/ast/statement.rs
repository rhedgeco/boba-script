use crate::parser::{Lexer, PError, PResult, Token};

use super::{Expr, Func, Node, While};

#[derive(Debug, Clone)]
pub enum Statement {
    Assign(Node<String>, Node<Expr>),
    LetAssign(Node<String>, Node<Expr>),
    Expr(Node<Expr>),
    Func(Node<Func>),
    While(Node<While>),
}

impl Statement {
    pub fn parse(tokens: &mut Lexer) -> PResult<Node<Self>> {
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
                let span = tokens.span(start..rhs.span().range().end);
                Ok(Node::new(span, Self::LetAssign(lhs, rhs)))
            }
            (Token::Ident(ident), span) => {
                let ident = Node::new(span.clone(), ident.to_string());
                tokens.next(); // consume ident

                match tokens.peek() {
                    Some(Err(error)) => Err(error),
                    None => Ok(Node::new(
                        ident.span().clone(),
                        Self::Expr(Node::new(ident.span().clone(), Expr::Var(ident))),
                    )),
                    Some(Ok((Token::Assign, _))) => {
                        tokens.next(); // consume assign
                        let rhs = Expr::parse(tokens)?;
                        tokens.expect_line_end()?;
                        let range = ident.span().range().start..rhs.span().range().end;
                        Ok(Node::new(tokens.span(range), Self::Assign(ident, rhs)))
                    }
                    Some(Ok(_)) => {
                        let lhs = Expr::parse_ident(ident, tokens)?;
                        let expr = Expr::parse_with_lhs(lhs, tokens)?;
                        tokens.expect_line_end()?;
                        Ok(Node::new(expr.span().clone(), Self::Expr(expr)))
                    }
                }
            }
            (Token::Fn, _) => {
                let func = Func::parse(tokens)?;
                Ok(Node::new(func.span().clone(), Self::Func(func)))
            }
            (Token::While, _) => {
                let r#while = While::parse(tokens)?;
                Ok(Node::new(r#while.span().clone(), Self::While(r#while)))
            }
            _ => {
                let expr = Expr::parse(tokens)?;
                tokens.expect_line_end()?;
                Ok(Node::new(expr.span().clone(), Self::Expr(expr)))
            }
        }
    }
}
