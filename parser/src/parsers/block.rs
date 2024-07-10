use boba_script_core::ast::StatementNode;

use crate::{
    error::ParseError,
    stream::{LineParser, TokenLine},
    PError, Token, TokenStream,
};

use super::statement;

pub enum Header<Data> {
    Complete(StatementNode<Data>),
    Incomplete(Data),
}

pub fn parse<T: TokenStream>(
    parser: &mut LineParser<T>,
) -> Result<Vec<StatementNode<T::Source>>, Vec<PError<T>>> {
    let header = parse_header(&mut parser.line())?;
    parse_with_header(header, parser)
}

pub fn parse_header<T: TokenStream>(
    line: &mut TokenLine<T>,
) -> Result<Header<T::Source>, Vec<PError<T>>> {
    line.parse_next_else(
        |token, line| match token {
            // INCOMPLETE CASE
            Some(Token::Colon) => {
                // capture colon source
                let source = line.token_source();

                // ensure end of line
                line.take_expect(None).map_err(|e| vec![e])?;

                // return incomplete
                Ok(Header::Incomplete(source))
            }

            // COMPLETE CASE
            Some(Token::FatArrow) => {
                let inline_source = line.token_source();
                Ok(Header::Complete(statement::parse_inline(
                    inline_source,
                    line,
                )?))
            }

            // FAILURE CASE
            token => Err(vec![ParseError::UnexpectedInput {
                expect: "':' or '=>'".into(),
                found: token,
                source: line.token_source(),
            }]),
        },
        // consume rest of line on error
        |errors| errors.consume_line(),
    )
}

pub fn parse_with_header<T: TokenStream>(
    header: Header<T::Source>,
    parser: &mut LineParser<T>,
) -> Result<Vec<StatementNode<T::Source>>, Vec<PError<T>>> {
    if let Header::Complete(statement) = header {
        return Ok(vec![statement]);
    }

    // consume the previous line
    parser.consume_line()?;

    // then check for an indent
    match parser.line().peek_next().map_err(|e| vec![e])? {
        // if there is an indent, continue on
        Some(Token::Indent) => {}
        // if its not an indent, then the block body is empty
        _ => return Ok(Vec::new()),
    }

    // consume all the statements in the body
    let mut body = Vec::new();
    let mut parse_errors = Vec::new();
    loop {
        // parse the next statement
        match statement::parse(parser) {
            Err(mut errors) => parse_errors.append(&mut errors),
            Ok(statement) => body.push(statement),
        }

        // consume the line
        if let Err(mut errors) = parser.consume_line() {
            parse_errors.append(&mut errors);
        }

        // check for dedent
        match parser.line().peek_next() {
            // if a dedent or none is found, consume it and break
            Ok(Some(Token::Dedent)) | Ok(None) => match parser.line().take_next() {
                Ok(Some(Token::Dedent)) | Ok(None) => break,
                _ => unreachable!(),
            },

            // if any other token is found, continue
            Ok(Some(_)) => continue,

            // if an error is found, consume the line
            Err(error) => {
                parse_errors.push(error);
                parser.line().consume_line(&mut parse_errors);
            }
        }
    }

    match parse_errors.is_empty() {
        // if there are errors, return those
        false => Err(parse_errors),
        // otherwise return the body
        true => Ok(body),
    }
}
