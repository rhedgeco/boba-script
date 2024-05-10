use std::ops::{Deref, DerefMut};

use crate::{
    lexer::{token::Span, Token},
    BobaError,
};

pub type TokenData = (Token, Span);

pub trait TokenIter: Iterator<Item = TokenData> {}
impl<T: Iterator<Item = TokenData> + Sized> TokenIter for T {}

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
    pub fn new(span: Span, item: T) -> Self {
        Self { span, item }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn into_inner(self) -> T {
        self.item
    }
}

pub struct ParseSource<'a> {
    iter: &'a mut dyn TokenIter,
}

impl<'a> ParseSource<'a> {
    pub fn new(iter: &'a mut dyn TokenIter) -> Self {
        Self { iter }
    }

    pub fn parse<T>(
        &mut self,
        parser: impl FnOnce(&mut NodeBuilder) -> Result<T, BobaError>,
    ) -> Result<Node<T>, BobaError> {
        // get first item and span start
        let (mut peeked, span_start) = match self.iter.next() {
            Some((token, span)) => {
                let span_start = span.start;
                (Some((token, span)), span_start)
            }
            None => (None, 0),
        };

        // create token server
        let mut builder = NodeBuilder {
            source: self.iter,
            peeked: &mut peeked,
            span: span_start..span_start,
        };

        // build the item
        let item = parser(&mut builder);

        // build node and return the item
        Ok(Node {
            span: builder.span,
            item: item?,
        })
    }
}

pub struct NodeBuilder<'a> {
    source: &'a mut dyn TokenIter,
    peeked: &'a mut Option<TokenData>,
    span: Span,
}

impl<'a> Iterator for NodeBuilder<'a> {
    type Item = TokenData;

    fn next(&mut self) -> Option<Self::Item> {
        let (token, span) = self.peeked.take()?;
        *self.peeked = self.source.next();
        self.span.end = span.end;
        Some((token, span))
    }
}

impl<'a> NodeBuilder<'a> {
    pub fn finish<T>(self, item: T) -> Node<T> {
        Node {
            item,
            span: self.span.clone(),
        }
    }

    pub fn span(&self) -> &Span {
        &self.span
    }

    pub fn peek(&self) -> Option<&TokenData> {
        self.peeked.as_ref()
    }

    pub fn has_next(&self) -> bool {
        self.peeked.is_some()
    }

    pub fn parse<T>(
        &mut self,
        parser: impl FnOnce(&mut NodeBuilder) -> Result<T, BobaError>,
    ) -> Result<Node<T>, BobaError> {
        // get first item and span start
        let span_start = match &self.peeked {
            Some((_, span)) => span.start,
            None => self.span.end,
        };

        // create token server
        let mut nested = NodeBuilder {
            source: self.source,
            peeked: self.peeked,
            span: span_start..span_start,
        };

        // build the item
        let item = parser(&mut nested);

        // update the current span
        self.span.end = nested.span.end;

        // build node and return the item
        Ok(Node {
            span: nested.span,
            item: item?,
        })
    }
}
