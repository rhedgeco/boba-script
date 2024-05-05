use std::iter::Peekable;

use logos::Span;

use crate::token::Token;

pub type Color = ariadne::Color;

// blanket token iterator impl
pub trait TokenIter: Iterator<Item = (Token, Span)> {}
impl<T: Iterator<Item = (Token, Span)>> TokenIter for T {}

pub struct ParserError {
    pub message: String,
    pub labels: Vec<ErrorLabel>,
}

pub struct ErrorLabel {
    pub message: String,
    pub color: Color,
    pub span: Span,
}

pub trait TokenParser {
    type Output;
    fn parse(tokens: &mut Peekable<impl TokenIter>) -> Result<Self::Output, ParserError>;
}
