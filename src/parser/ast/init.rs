use ariadne::Span;

use crate::{
    cache::CacheSpan,
    parser::{Lexer, PError, PResult, Token},
};

use super::{Expr, Node};

#[derive(Debug, Clone, Copy)]
pub enum InitStyle {
    Let,
    Static,
    Const,
}

#[derive(Debug, Clone)]
pub struct Init<Data> {
    pub style: Node<Data, InitStyle>,
    pub ident: Node<Data, String>,
    pub expr: Node<Data, Expr<Data>>,
}

impl Init<CacheSpan> {
    pub fn parse(tokens: &mut Lexer) -> PResult<CacheSpan, Node<CacheSpan, Self>> {
        let style = match tokens.expect_next("'let', 'const', or 'static'")? {
            (Token::Let, span) => Node::new(span, InitStyle::Let),
            (Token::Const, span) => Node::new(span, InitStyle::Const),
            (Token::Static, span) => Node::new(span, InitStyle::Static),
            (token, span) => {
                return Err(PError::UnexpectedToken {
                    expected: format!("'let', 'const', or 'static'"),
                    found: format!("'{token}'"),
                    data: span,
                })
            }
        };

        let ident = match tokens.expect_next("identifier")? {
            (Token::Ident(ident), span) => Node::new(span, ident.to_string()),
            (token, span) => {
                return Err(PError::UnexpectedToken {
                    expected: format!("identifier"),
                    found: format!("'{token}'"),
                    data: span,
                })
            }
        };

        match tokens.expect_next("=")? {
            (Token::Assign, _) => (),
            (token, span) => {
                return Err(PError::UnexpectedToken {
                    expected: format!("="),
                    found: format!("'{token}'"),
                    data: span,
                })
            }
        }

        let expr = Expr::parse(tokens)?;
        tokens.expect_line_end()?;

        Ok(Node::new(
            tokens.span(style.data().start()..expr.data().end()),
            Self { style, ident, expr },
        ))
    }
}
