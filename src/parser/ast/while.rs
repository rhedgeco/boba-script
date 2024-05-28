use crate::{
    cache::CacheSpan,
    parser::{Lexer, PError, PResult, Token},
};

use super::{Expr, Node, Statement};

#[derive(Debug, Clone)]
pub struct While<Data> {
    pub cond: Node<Data, Expr<Data>>,
    pub body: Vec<Node<Data, Statement<Data>>>,
}

impl While<CacheSpan> {
    pub fn parse(tokens: &mut Lexer) -> PResult<CacheSpan, Node<CacheSpan, Self>> {
        // capture while token
        let start = match tokens.expect_next("'while'")? {
            (Token::While, span) => span.range().start,
            (token, span) => {
                return Err(PError::UnexpectedToken {
                    expected: format!("'while'"),
                    found: format!("'{token}'"),
                    data: span,
                })
            }
        };

        // capture condition
        let cond = Expr::parse(tokens)?;

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
            tokens.span(start..end),
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
        tokens.expect_line_end()?;
        let statement = Node::new(expr.data().clone(), Statement::Expr(expr));
        output.body.push(statement);
        Ok(output)
    }
}
