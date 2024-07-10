use thiserror::Error;

use crate::{Token, TokenStream};

pub type PError<T> = ParseError<<T as TokenStream>::Source, <T as TokenStream>::Error>;

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ParseError<Source, TokenError> {
    TokenError {
        error: TokenError,
        source: Source,
    },
    UnexpectedInput {
        expect: String,
        found: Option<Token>,
        source: Source,
    },
    UnclosedBrace {
        open: Source,
        end: Source,
    },
    InlineError {
        source: Source,
    },
    EmptyBlock {
        source: Source,
    },
}
