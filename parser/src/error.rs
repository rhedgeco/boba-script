use thiserror::Error;

use crate::{stream::Source, Token, TokenStream};

pub type PError<T> =
    ParseError<<<T as TokenStream>::Source as Source>::Span, <T as TokenStream>::Error>;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ParseError<Span, TokenError> {
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
