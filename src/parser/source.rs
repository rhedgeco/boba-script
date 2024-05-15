use logos::{Logos, SpannedIter};

use crate::lexer::{token::Span, Token};

use super::{node::NodeBuilder, ParserSource};

pub struct TokenSource<'source> {
    source: &'source str,
    iter: SpannedIter<'source, Token<'source>>,
    peek: Option<(Token<'source>, Span)>,
    pos: usize,
}

impl<'source> TokenSource<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            source,
            iter: Token::lexer(source).spanned(),
            peek: None,
            pos: 0,
        }
    }
}

impl<'source> ParserSource<'source> for TokenSource<'source> {
    fn pos(&self) -> usize {
        self.pos
    }

    fn source(&self) -> &str {
        &self.source
    }

    fn peek(&mut self) -> Option<&(Token<'source>, Span)> {
        if self.peek.is_none() {
            self.peek = self.take();
        }

        self.peek.as_ref()
    }

    fn take(&mut self) -> Option<(Token<'source>, Span)> {
        match self.peek.take() {
            Some(next) => Some(next),
            None => {
                let (token, span) = self.iter.next()?;
                let token = token.expect("unhandled invalid token");
                Some((token, span))
            }
        }
    }

    fn node_builder<'a>(&'a mut self) -> NodeBuilder<'a, 'source, Self> {
        NodeBuilder::new(self)
    }
}
