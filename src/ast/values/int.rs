use std::ops::{Deref, DerefMut};

use crate::{
    ast::Spanned,
    parser::{PError, TokenSource},
    token::Span,
    Token,
};

#[derive(Debug)]
pub struct Int {
    span: Span,
    val: i64,
}

impl Spanned for Int {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl DerefMut for Int {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl Deref for Int {
    type Target = i64;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl Int {
    pub fn new(val: i64, span: Span) -> Self {
        Self { span, val }
    }

    pub fn value(&self) -> i64 {
        self.val
    }

    pub fn parse_str(str: impl AsRef<str>, span: Span) -> Result<Self, PError> {
        match str.as_ref().parse() {
            Ok(value) => Ok(Self::new(value, span)),
            Err(error) => Err(PError::ParseIntError { error, span }),
        }
    }

    pub fn parse(source: &mut TokenSource) -> Result<Self, PError> {
        match source.take() {
            Some((Token::Int(str), span)) => Self::parse_str(str, span),
            Some((token, span)) => Err(PError::UnexpectedToken {
                expect: "int".into(),
                found: format!("'{token}'"),
                span,
            }),
            None => Err(PError::UnexpectedEnd {
                expect: "int".into(),
                pos: source.pos(),
            }),
        }
    }
}
