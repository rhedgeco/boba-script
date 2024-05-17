use std::ops::{Deref, DerefMut};

use crate::token::Span;

pub trait Spanned {
    fn span(&self) -> Span;
}

#[derive(Debug)]
pub struct Spanner<T> {
    span: Span,
    item: T,
}

impl<T> Spanned for Spanner<T> {
    fn span(&self) -> Span {
        self.span.clone()
    }
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

impl<T> Spanner<T> {
    pub fn new(span: Span, item: T) -> Self {
        Self { span, item }
    }

    pub fn into_inner(self) -> T {
        self.item
    }
}
