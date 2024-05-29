use crate::{
    cache::CacheSpan,
    parser::{Lexer, PError, PResult, Token},
};

use super::{Expr, Func, Node, While};

#[derive(Debug, Clone)]
pub enum Statement<Data> {
    Assign(Node<String, Data>, Node<Expr<Data>, Data>),
    LetAssign(Node<String, Data>, Node<Expr<Data>, Data>),
    Expr(Node<Expr<Data>, Data>),
    Func(Node<Func<Data>, Data>),
    While(Node<While<Data>, Data>),
}

impl Statement<CacheSpan> {
    pub fn parse(tokens: &mut Lexer) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
        match tokens.expect_peek("assignment or expression")? {
            (Token::Let, span) => {
                let start = span.range().start;
                tokens.next(); // consume let

                // capture lhs variable
                let lhs = match tokens.expect_next("identifier")? {
                    (Token::Ident(var), span) => Node::new(var.to_string(), span),
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
                Ok(Node::new(Self::LetAssign(lhs, rhs), span))
            }
            (Token::Ident(ident), span) => {
                let ident = Node::new(ident.to_string(), span);
                tokens.next(); // consume ident

                match tokens.peek() {
                    Some(Err(error)) => Err(error),
                    None => {
                        let span = *ident.data();
                        let expr = Node::new(Expr::Var(ident), span);
                        Ok(Node::new(Self::Expr(expr), span))
                    }
                    Some(Ok((Token::Assign, _))) => {
                        tokens.next(); // consume assign
                        let rhs = Expr::parse(tokens)?;
                        tokens.expect_line_end()?;
                        let range = ident.data().range().start..rhs.data().range().end;
                        Ok(Node::new(Self::Assign(ident, rhs), tokens.span(range)))
                    }
                    Some(Ok(_)) => {
                        let lhs = Expr::parse_ident(ident, tokens)?;
                        let expr = Expr::parse_with_lhs(lhs, tokens)?;
                        tokens.expect_line_end()?;
                        let span = *expr.data();
                        Ok(Node::new(Self::Expr(expr), span))
                    }
                }
            }
            (Token::Fn, _) => {
                let func = Func::parse(tokens)?;
                let span = *func.data();
                Ok(Node::new(Self::Func(func), span))
            }
            (Token::While, _) => {
                let r#while = While::parse(tokens)?;
                let span = *r#while.data();
                Ok(Node::new(Self::While(r#while), span))
            }
            _ => {
                let expr = Expr::parse(tokens)?;
                let span = *expr.data();
                tokens.expect_line_end()?;
                Ok(Node::new(Self::Expr(expr), span))
            }
        }
    }
}
