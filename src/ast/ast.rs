use std::{
    iter::Peekable,
    ops::{Deref, DerefMut},
};

use ariadne::{Label, Report, ReportKind, Source};
use logos::Span;

use crate::token::Token;

pub type Color = ariadne::Color;

// blanket token iterator impl
pub trait TokenIter: Iterator<Item = (Token, Span)> {}
impl<T: Iterator<Item = (Token, Span)>> TokenIter for T {}

pub struct BobaError {
    pub message: String,
    pub labels: Vec<ErrorLabel>,
}

impl BobaError {
    pub fn report(self, id: impl AsRef<str>, source: Source<impl AsRef<str> + Clone>) {
        let id = id.as_ref();
        let mut report = Report::build(ReportKind::Error, "shell", 0)
            .with_code(1)
            .with_message(self.message);

        for label in self.labels {
            report.add_label(
                Label::new((id, label.span))
                    .with_color(label.color)
                    .with_message(label.message),
            )
        }

        report.finish().eprint((id, source.clone())).unwrap();
    }
}

pub struct ErrorLabel {
    pub message: String,
    pub color: Color,
    pub span: Span,
}

pub trait TokenParser {
    type Output;
    fn parse(tokens: &mut Peekable<impl TokenIter>) -> Result<Node<Self::Output>, BobaError>;
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
