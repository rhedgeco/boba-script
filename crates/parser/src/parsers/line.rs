use crate::{error::PError, ParseError, Token, TokenLine, TokenStream};

pub enum CloseType {
    SemiColon,
    EndLine,
}

pub fn parse_close<T: TokenStream>(line: &mut TokenLine<T>) -> Result<CloseType, Vec<PError<T>>> {
    line.take_guard_else(
        |token, line| match token {
            // END LINE CASE
            None => Ok(CloseType::EndLine),

            // SEMICOLON CASE
            Some(Token::SemiColon) => {
                // ensure nothing comes after semicolon
                line.take_exact(None).map_err(|e| vec![e])?;
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
