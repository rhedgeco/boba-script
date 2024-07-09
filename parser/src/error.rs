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
    UnclosedBrace {
        open: Span,
        end: Span,
    },
}
