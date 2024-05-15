use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

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
    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn into_inner(self) -> T {
        self.item
    }
}

pub struct NodeBuilder<'a, 'source, P: ParserSource<'source>> {
    _lt: PhantomData<&'source ()>,
    span: Option<Span>,
    source: &'a mut P,
}

impl<'a, 'source, P: ParserSource<'source>> NodeBuilder<'a, 'source, P> {
    pub(super) fn new(source: &'a mut P) -> Self {
        Self {
            _lt: PhantomData,
            span: None,
            source,
        }
    }

    pub fn build<T>(self, item: T) -> Node<T> {
        let span = match self.span {
            None => self.source.pos()..self.source.pos(),
            Some(span) => span,
        };

        Node { span, item }
    }
}

impl<'a, 'source, P: ParserSource<'source>> ParserSource<'source> for NodeBuilder<'a, 'source, P> {
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

    fn node_builder(&mut self) -> NodeBuilder<'_, 'source, Self> {
        NodeBuilder::new(self)
    }
}
