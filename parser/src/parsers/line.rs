use crate::{error::ParseError, stream::TokenLine, PError, Token, TokenStream};

pub enum CloseType {
    SemiColon,
    EndLine,
}

pub fn parse_close<T: TokenStream>(parser: &mut TokenLine<T>) -> Result<CloseType, Vec<PError<T>>> {
    parser.parse_next_else(
        |token, line| match token {
            // END LINE CASE
            None => Ok(CloseType::EndLine),

            // SEMICOLON CASE
            Some(Token::SemiColon) => {
                // ensure nothing comes after semicolon
                line.take_expect(None).map_err(|e| vec![e])?;
                Ok(CloseType::SemiColon)
            }

            // FAILURE CASE
            Some(token) => Err(vec![ParseError::UnexpectedInput {
                expect: "';' or end of line".into(),
                found: Some(token),
                source: line.token_source(),
            }]),
        },
        // consume the rest of the line on error
        |errors| errors.consume_line(),
    )
}
