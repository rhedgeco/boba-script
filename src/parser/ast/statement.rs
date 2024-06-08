use crate::{
    cache::CacheSpan,
    parser::{Lexer, PResult, Token},
};

use super::{Expr, Init, Node};

#[derive(Debug, Clone)]
pub enum Statement<Data> {
    Init(Node<Data, Init<Data>>),
    Expr(Node<Data, Expr<Data>>),
}

impl Statement<CacheSpan> {
    pub fn parse(tokens: &mut Lexer) -> PResult<CacheSpan, Node<CacheSpan, Self>> {
        match tokens.expect_peek("assignment or expression")? {
            (Token::Let, _) | (Token::Const, _) | (Token::Static, _) => {
                let init = Init::parse(tokens)?;
                Ok(Node::new(init.data().clone(), Self::Init(init)))
            }
            _ => {
                let expr = Expr::parse(tokens)?;
                tokens.expect_line_end()?;
                Ok(Node::new(expr.data().clone(), Self::Expr(expr)))
            }
        }
    }
}
