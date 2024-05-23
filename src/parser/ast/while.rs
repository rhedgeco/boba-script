use crate::parser::{PError, PResult, Token, TokenLine};

use super::{Expr, Node, Statement};

#[derive(Debug, Clone)]
pub struct While {
    pub cond: Node<Expr>,
    pub body: Vec<Node<Statement>>,
}

impl While {
    pub fn parse(tokens: &mut TokenLine) -> PResult<Node<Self>> {
        // capture while token
        let start = match tokens.expect_next("'while'")? {
            (Token::While, span) => span.start,
            (token, span) => {
                return Err(PError::UnexpectedToken {
                    expected: format!("'while'"),
                    found: format!("'{token}'"),
                    span,
                })
            }
        };

        // capture condition
        let cond = Expr::parse(tokens)?;

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
                cond,
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
