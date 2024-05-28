use std::ops::{Deref, DerefMut};

use crate::cache::CacheSpan;

#[derive(Debug, Clone)]
pub struct Node<T> {
    span: CacheSpan,
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
    pub fn new(span: CacheSpan, item: T) -> Self {
        Self { span, item }
    }

    pub fn span(&self) -> &CacheSpan {
        &self.span
    }

    pub fn into_item(self) -> T {
        self.item
    }

    pub fn into_span(self) -> CacheSpan {
        self.span
    }

    pub fn into_parts(self) -> (CacheSpan, T) {
        (self.span, self.item)
    }
}
