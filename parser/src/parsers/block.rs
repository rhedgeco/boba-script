use std::mem::replace;

use boba_script_core::ast::StatementNode;

use crate::{error::PError, stream::SourceSpan, ParseError, Token, TokenLine, TokenStream};

use super::statement::{self, StatementParser, StatementType};

pub fn start_parsing<T: TokenStream>(
    line: &mut TokenLine<T>,
) -> Result<BlockParser<T::Source>, Vec<PError<T>>> {
    line.take_guard_else(
        |token, line| match token {
            // check for leading block colon
            Some(Token::Colon) => {
                // ensure end of line
                line.take_exact(None).map_err(|e| vec![e])?;

                // build block parser
                Ok(BlockParser {
                    pending: None,
                    body: Vec::new(),
                    complete: false,
                })
            }

            // otherwise return an error
            token => Err(vec![ParseError::UnexpectedInput {
                expect: "':'".into(),
                found: token,
                source: line.token_source(),
            }]),
        },
        |errors| errors.consume_line(),
    )
}

pub struct BlockParser<Source: SourceSpan> {
    pending: Option<Box<StatementParser<Source>>>,
    body: Vec<StatementNode<Source>>,
    complete: bool,
}

impl<Source: SourceSpan> BlockParser<Source> {
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    pub fn parse_line<T: TokenStream<Source = Source>>(
        &mut self,
        line: &mut TokenLine<T>,
    ) -> Result<Option<Vec<StatementNode<Source>>>, Vec<PError<T>>> {
        // check for complete block
        if self.complete {
            return Ok(None);
        }

        // if there is nothing pending and no body, then it is the start
        if self.pending.is_none() && self.body.is_empty() {
            match line.peek_token() {
                // consume indent if found
                Some(Ok(Token::Indent)) => line.consume_token(),

                // otherwise return an empty body
                _ => return Ok(Some(Vec::new())),
            }
        }

        // parse any pending statements
        match self.pending.take() {
            Some(mut parser) => match parser.parse_line(line)? {
                Some(statement) => self.body.push(statement),
                None => self.pending = Some(parser),
            },

            // if no more statements are pending check for dedent
            None => match line.peek_token() {
                // if we find a dedent, then end parsing and return the data
                Some(Ok(Token::Dedent)) => {
                    let body = replace(&mut self.body, Vec::new());
                    return Ok(Some(body));
                }

                // if we find anything else, parse the line as a statement
                _ => match statement::start_parsing(line)? {
                    StatementType::MultiLine(parser) => self.pending = Some(Box::new(parser)),
                    StatementType::SingleLine(statement) => self.body.push(statement),
                },
            },
        }

        // return nothing if incomplete
        Ok(None)
    }
}
