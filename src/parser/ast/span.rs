use std::ops::{Deref, DerefMut};

use crate::parser::Span;

pub trait Spanned {
    fn span(&self) -> Span;
}

impl<S: Spanned> Spanned for Box<S> {
    fn span(&self) -> Span {
        self.deref().span()
    }
}

pub struct SpanPrefix<S: Spanned> {
    offset: usize,
    item: S,
}

impl<S: Spanned> DerefMut for SpanPrefix<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<S: Spanned> Deref for SpanPrefix<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<S: Spanned> Spanned for SpanPrefix<S> {
    fn span(&self) -> Span {
        let span = self.item.span();
        span.start - self.offset..span.end
    }
}

impl<S: Spanned> SpanPrefix<S> {
    pub fn new(offset: usize, item: S) -> Self {
        Self { offset, item }
    }

    pub fn into_inner(self) -> S {
        self.item
    }
}

pub struct Spanner<T> {
    span: Span,
    item: T,
}

impl<T> DerefMut for Spanner<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}

impl<T> Deref for Spanner<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T> Spanned for Spanner<T> {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl<T> Spanner<T> {
    pub fn new(span: Span, item: T) -> Self {
        Self { span, item }
    }

    pub fn into_inner(self) -> T {
        self.item
    }
}
