use crate::parser::{PError, PResult, Token, TokenLine};

use super::{Expr, Node, Statement};

#[derive(Debug, Clone)]
pub struct Func {
    pub ident: Node<String>,
    pub body: Vec<Node<Statement>>,
}

impl Func {
    pub fn parse(tokens: &mut TokenLine) -> PResult<Node<Self>> {
        // capture fn token
        let start = match tokens.expect_next("'fn'")? {
            (Token::Fn, span) => span.start,
            (token, span) => {
                return Err(PError::UnexpectedToken {
                    expected: format!("'fn'"),
                    found: format!("'{token}'"),
                    span,
                })
            }
        };

        // capture ident token
        let ident = match tokens.expect_next("function name")? {
            (Token::Ident(name), span) => Node::new(span, name.to_string()),
            (token, span) => {
                return Err(PError::UnexpectedToken {
                    expected: format!("function name"),
                    found: format!("'{token}'"),
                    span,
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
                    span,
                })
            }
        };

        // capture close paren
        match tokens.expect_next("')'")? {
            (Token::CloseParen, _) => (),
            (token, span) => {
                return Err(PError::UnexpectedToken {
                    expected: format!("')'"),
                    found: format!("'{token}'"),
                    span,
                })
            }
        };

        // capture colon token
        let end = match tokens.expect_next("':'")? {
            (Token::Colon, span) => span.end,
            (token, span) => {
                return Err(PError::UnexpectedToken {
                    expected: format!("':'"),
                    found: format!("'{token}'"),
                    span,
                })
            }
        };

        // create output
        let mut output = Node::new(
            start..end,
            Self {
                ident,
                body: Vec::new(),
            },
        );

        // return early if end of line is found
        if let None = tokens.peek() {
            return Ok(output);
        }

        // capture single expression
        let expr = Expr::parse(tokens)?;
        tokens.expect_end()?;
        let statement = Node::new(expr.span().clone(), Statement::Expr(expr));
        output.body.push(statement);
        Ok(output)
    }
}
