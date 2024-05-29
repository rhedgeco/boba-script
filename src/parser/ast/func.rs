use crate::{
    cache::CacheSpan,
    parser::{Lexer, PError, PResult, Token},
};

use super::{Node, Statement};

#[derive(Debug, Clone)]
pub struct Func<Data> {
    pub ident: Node<String, Data>,
    pub params: Vec<Node<String, Data>>,
    pub body: Vec<Node<Statement<Data>, Data>>,
}

impl Func<CacheSpan> {
    pub fn parse(tokens: &mut Lexer) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
        // capture fn token
        let start = match tokens.expect_next("'fn'")? {
            (Token::Fn, span) => span.range().start,
            (token, span) => {
                return Err(PError::UnexpectedToken {
                    expected: format!("'fn'"),
                    found: format!("'{token}'"),
                    data: span,
                })
            }
        };

        // capture ident token
        let ident = match tokens.expect_next("function name")? {
            (Token::Ident(name), span) => Node::new(name.to_string(), span),
            (token, span) => {
                return Err(PError::UnexpectedToken {
                    expected: format!("function name"),
                    found: format!("'{token}'"),
                    data: span,
                })
            }
        };

        // capture open paren
        match tokens.expect_next("'('")? {
            (Token::OpenParen, _) => (),
            (token, span) => {
                return Err(PError::UnexpectedToken {
                    expected: format!("'('"),
                    found: format!("'{token}'"),
                    data: span,
                })
            }
        };

        // capture parameters
        let mut params = Vec::new();
        while let (Token::Ident(param), span) = tokens.expect_peek("parameter or ')'")? {
            // push parameter
            params.push(Node::new(param.to_string(), span));
            tokens.next(); // consume ident

            // capture comma
            match tokens.expect_peek("',' or ')'")? {
                (Token::Comma, _) => {
                    tokens.next(); // consume comma
                }
                // if no comma found, then there are no more params
                _ => break,
            }
        }

        // capture close paren
        match tokens.expect_next("')'")? {
            (Token::CloseParen, _) => (),
            (token, span) => {
                return Err(PError::UnexpectedToken {
                    expected: format!("')'"),
                    found: format!("'{token}'"),
                    data: span,
                })
            }
        };

        // capture colon token
        let end = match tokens.expect_next("':'")? {
            (Token::Colon, span) => span.range().end,
            (token, span) => {
                return Err(PError::UnexpectedToken {
                    expected: format!("':'"),
                    found: format!("'{token}'"),
                    data: span,
                })
            }
        };

        // create output
        let mut output = Node::new(
            Self {
                ident,
                params,
                body: Vec::new(),
            },
            tokens.span(start..end),
        );

        // return early if end of line is found
        match tokens.peek() {
            None => return Ok(output),
            Some(Err(err)) => return Err(err),
            Some(Ok((token, _))) => match token {
                Token::Newline => return Ok(output),
                _ => (),
            },
        }

        // capture single statement
        let statement = Statement::parse(tokens)?;
        tokens.expect_line_end()?;
        output.body.push(statement);
        Ok(output)
    }
}
