use std::ops::{Deref, DerefMut};

use crate::lexer::{token::Span, Token};

use super::ParserSource;

#[derive(Debug)]
pub struct Node<T> {
    span: Span,
    item: T,
}

impl<T> DerefMut for Node<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<T> Deref for Node<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T> Node<T> {
    pub fn build(span: Span, item: T) -> Self {
        Self { span, item }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn into_inner(self) -> T {
        self.item
    }
}

pub struct NodeBuilder<'a, 'source> {
    source: &'a mut dyn ParserSource<'source>,
    span: Option<Span>,
}

impl<'a, 'source> NodeBuilder<'a, 'source> {
    pub(super) fn new(source: &'a mut impl ParserSource<'source>) -> Self {
        Self { source, span: None }
    }

    pub fn build<T>(self, item: T) -> Node<T> {
        let span = match self.span {
            None => self.source.pos()..self.source.pos(),
            Some(span) => span,
        };

        Node::build(span, item)
    }
}

impl<'a, 'source> ParserSource<'source> for NodeBuilder<'a, 'source> {
    fn pos(&self) -> usize {
        self.source.pos()
    }

    fn source(&self) -> &str {
        self.source.source()
    }

    fn peek(&mut self) -> Option<&(Token<'source>, Span)> {
        self.source.peek()
    }

    fn take(&mut self) -> Option<(Token<'source>, Span)> {
        let (token, span) = self.source.take()?;

        // update span tracking
        match &mut self.span {
            Some(node_span) => node_span.end = span.end,
            None => self.span = Some(span.clone()),
        }

        Some((token, span))
    }

    fn node_builder(&mut self) -> NodeBuilder<'_, 'source> {
        NodeBuilder::new(self)
    }
}
