use std::ops::{Deref, DerefMut};

use crate::{
    ast::Spanned,
    parser::{report::PError, TokenSource},
    token::Span,
    Token,
};

#[derive(Debug)]
pub struct StringSpan {
    span: Span,
    val: String,
}

impl Spanned for StringSpan {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl DerefMut for StringSpan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl Deref for StringSpan {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl StringSpan {
    pub fn new(val: String, span: Span) -> Self {
        Self { span, val }
    }

    pub fn value(&self) -> String {
        self.val.clone()
    }

    pub fn parse(source: &mut TokenSource) -> Result<Self, PError> {
        match source.take() {
            Some((Token::String(value), span)) => Ok(Self::new(value.into(), span)),
            Some((token, span)) => Err(PError::UnexpectedToken {
                expect: "String".into(),
                found: format!("'{token}'"),
                span,
            }),
            None => Err(PError::UnexpectedEnd {
                expect: "String".into(),
                pos: source.pos(),
            }),
        }
    }
}
