use crate::lexer::{token::Span, Token};

use super::node::NodeBuilder;

pub trait ParserSource<'source>: Sized {
    fn pos(&self) -> usize;
    fn source(&self) -> &str;
    fn peek(&mut self) -> Option<&(Token<'source>, Span)>;
    fn take(&mut self) -> Option<(Token<'source>, Span)>;
    fn node_builder(&mut self) -> NodeBuilder<'_, 'source, Self>;
}
