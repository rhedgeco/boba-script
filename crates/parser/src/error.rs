use thiserror::Error;

use crate::{Token, TokenStream};

pub type PError<S> = ParseError<<S as TokenStream>::Source, <S as TokenStream>::Error>;

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
        inline_source: Source,
        block_source: Source,
    },
    EmptyBlock {
        source: Source,
    },
}
