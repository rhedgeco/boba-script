use std::{
    iter::Peekable,
    ops::{Deref, DerefMut},
};

use logos::Span;

use crate::token::Token;

pub type Color = ariadne::Color;

// blanket token iterator impl
pub trait TokenIter: Iterator<Item = (Token, Span)> {}
impl<T: Iterator<Item = (Token, Span)>> TokenIter for T {}

pub struct ParserError {
    pub message: String,
    pub labels: Vec<ErrorLabel>,
}

pub struct ErrorLabel {
    pub message: String,
    pub color: Color,
    pub span: Span,
}

pub trait TokenParser {
    type Output;
    fn parse(tokens: &mut Peekable<impl TokenIter>) -> Result<Node<Self::Output>, ParserError>;
}

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
}
