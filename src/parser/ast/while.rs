use crate::{
    cache::CacheSpan,
    parser::{Lexer, PError, PResult, Token},
};

use super::{Expr, Node, Statement};

#[derive(Debug, Clone)]
pub struct While<Data> {
    pub cond: Node<Expr<Data>, Data>,
    pub body: Vec<Node<Statement<Data>, Data>>,
}

impl While<CacheSpan> {
    pub fn parse(tokens: &mut Lexer) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
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
            Self {
                cond,
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

        // capture single expression
        let expr = Expr::parse(tokens)?;
        tokens.expect_line_end()?;
        let span = *expr.data();
        let statement = Node::new(Statement::Expr(expr), span);
        output.body.push(statement);
        Ok(output)
    }
}
