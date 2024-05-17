use std::ops::{Deref, DerefMut};

use crate::{
    ast::Spanned,
    parser::{PError, TokenSource},
    token::Span,
    Token,
};

#[derive(Debug)]
pub struct Bool {
    span: Span,
    val: bool,
}

impl Spanned for Bool {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl DerefMut for Bool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl Deref for Bool {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl Bool {
    pub fn new(val: bool, span: Span) -> Self {
        Self { span, val }
    }

    pub fn value(&self) -> bool {
        self.val
    }

    pub fn parse(source: &mut TokenSource) -> Result<Self, PError> {
        match source.take() {
            Some((Token::Bool(value), span)) => Ok(Self::new(value, span)),
            Some((token, span)) => Err(PError::UnexpectedToken {
                expect: "bool".into(),
                found: format!("'{token}'"),
                span,
            }),
            None => Err(PError::UnexpectedEnd {
                expect: "bool".into(),
                pos: source.pos(),
            }),
        }
    }
}
