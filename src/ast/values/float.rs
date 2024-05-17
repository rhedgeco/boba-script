use std::ops::{Deref, DerefMut};

use crate::{
    ast::Spanned,
    parser::{PError, TokenSource},
    token::Span,
    Token,
};

#[derive(Debug)]
pub struct Float {
    span: Span,
    val: f64,
}

impl Spanned for Float {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl DerefMut for Float {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.val
    }
}

impl Deref for Float {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.val
    }
}

impl Float {
    pub fn new(val: f64, span: Span) -> Self {
        Self { span, val }
    }

    pub fn value(&self) -> f64 {
        self.val
    }

    pub fn parse_str(str: impl AsRef<str>, span: Span) -> Result<Self, PError> {
        match str.as_ref().parse() {
            Ok(value) => Ok(Self::new(value, span)),
            Err(error) => Err(PError::ParseFloatError { error, span }),
        }
    }

    pub fn parse(source: &mut TokenSource) -> Result<Self, PError> {
        match source.take() {
            Some((Token::Float(str), span)) => Self::parse_str(str, span),
            Some((token, span)) => Err(PError::UnexpectedToken {
                expect: "float".into(),
                found: format!("'{token}'"),
                span,
            }),
            None => Err(PError::UnexpectedEnd {
                expect: "float".into(),
                pos: source.pos(),
            }),
        }
    }
}
