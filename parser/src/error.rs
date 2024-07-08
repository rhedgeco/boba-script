use std::fmt::Display;

use thiserror::Error;

use crate::{stream::Source, Token, TokenStream};

pub type ParseError<T> =
    SpanParseError<<<T as TokenStream>::Source as Source>::Span, <T as TokenStream>::Error>;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum SpanParseError<Span, TokenError> {
    TokenError {
        error: TokenError,
        span: Span,
    },
    UnexpectedInput {
        expect: String,
        found: Option<Token>,
        span: Span,
    },
}

impl<Span, TokenError: Display> Display for SpanParseError<Span, TokenError> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpanParseError::TokenError { error, span: _ } => write!(f, "{error}"),
            SpanParseError::UnexpectedInput {
                expect,
                found,
                span: _,
            } => match found {
                Some(token) => write!(f, "expected {expect}, found {token}"),
                None => write!(f, "expected {expect}, found end of input"),
            },
        }
    }
}
