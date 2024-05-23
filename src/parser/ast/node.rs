use std::ops::{Deref, DerefMut};

use crate::parser::Span;

#[derive(Debug, Clone)]
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

    pub fn into_item(self) -> T {
        self.item
    }

    pub fn into_span(self) -> Span {
        self.span
    }

    pub fn into_parts(self) -> (Span, T) {
        (self.span, self.item)
    }
}
