use crate::{error::ParseError, PError, Token, TokenParser, TokenStream};

pub enum CloseType {
    Semicolon,
    LineEnd,
}

pub fn parse_end<T: TokenStream>(parser: &mut TokenParser<T>) -> Result<(), Vec<PError<T>>> {
    parser.parse_next_else(
        |token, parser| match token {
            None => Ok(()),
            token => Err(vec![ParseError::UnexpectedInput {
                expect: "end of line".into(),
                found: token,
                span: parser.token_span(),
            }]),
        },
        |parser| {
            parser.consume_until(|t| matches!(t, Token::Newline));
        },
    )
}

pub fn parse_close<T: TokenStream>(
    parser: &mut TokenParser<T>,
) -> Result<CloseType, Vec<PError<T>>> {
    parser.parse_next_else(
        |token, parser| match token {
            Some(Token::SemiColon) => Ok(CloseType::Semicolon),
            None => Ok(CloseType::LineEnd),
            token => Err(vec![ParseError::UnexpectedInput {
                expect: "';' or end of line".into(),
                found: token,
                span: parser.token_span(),
            }]),
        },
        |parser| {
            parser.consume_until(|t| matches!(t, Token::Newline | Token::SemiColon));
        },
    )
}
