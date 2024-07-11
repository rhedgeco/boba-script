use boba_script_core::ast::StatementNode;

use crate::{
    error::ParseError,
    stream::{SpanSource, TokenLine},
    PError, Token, TokenStream,
};

use super::statement::{self, StatementParser};

pub fn start_parsing<T: TokenStream>(
    line: &mut TokenLine<T>,
) -> Result<State<T::Source, T::Error>, Vec<PError<T>>> {
    line.parse_next_else(
        |token, line| match token {
            // INLINE CASE
            Some(Token::FatArrow) => {
                let inline_source = line.token_source();
                let statement = statement::parse_inline(inline_source, line)?;
                Ok(State::Complete(vec![statement]))
            }

            // MULTI LINE CASE
            Some(Token::Colon) => {
                // get block source
                let block_source = line.token_source();

                // ensure end of line
                line.take_expect(None).map_err(|e| vec![e])?;

                // build block parser
                Ok(State::Incomplete(BlockParser {
                    pending: Vec::new(),
                    errors: Vec::new(),
                    body: Vec::new(),
                    source: block_source,
                }))
            }

            // FAILURE CASE
            token => Err(vec![ParseError::UnexpectedInput {
                expect: "':' or '=>'".into(),
                found: token,
                source: line.token_source(),
            }]),
        },
        |errors| errors.consume_line(),
    )
}

pub enum State<Source: SpanSource, Error> {
    Complete(Vec<StatementNode<Source>>),
    Incomplete(BlockParser<Source, Error>),
}

pub struct BlockParser<Source: SpanSource, Error> {
    pending: Vec<StatementParser<Source, Error>>,
    errors: Vec<ParseError<Source, Error>>,
    body: Vec<StatementNode<Source>>,
    source: Source,
}

impl<Source: SpanSource, Error> BlockParser<Source, Error> {
    pub fn source(&self) -> Source {
        self.source.clone()
    }

    pub fn parse_line<T: TokenStream<Source = Source, Error = Error>>(
        mut self,
        line: &mut TokenLine<T>,
    ) -> Result<State<Source, Error>, Vec<PError<T>>> {
        // if the body is empty, ensure that it starts with an indent token
        if self.body.is_empty() {
            line.parse_peek(|peeker| match peeker.token() {
                // consume indent if found
                Some(Token::Indent) => {
                    peeker.consume();
                    Ok(())
                }

                // otherwise produce an empty body error
                _ => Err(vec![ParseError::EmptyBlock {
                    source: self.source.clone(),
                }]),
            })?;
        }

        // parse any pending statements
        let state = match self.pending.pop() {
            Some(parser) => parser.parse_line(line),

            // if no more statements are pending check for dedent
            None => match line.peek_next() {
                // if we find a dedent, then end parsing and return the data
                Ok(Some(Token::Dedent)) => match self.errors.is_empty() {
                    true => return Ok(State::Complete(self.body)),
                    false => return Err(self.errors),
                },

                // if we find any other token, parse the line as a statement
                Ok(_) => statement::start_parsing(line),

                // if we find an error, store it, consume the line, and return incomplete
                Err(error) => {
                    self.errors.push(error);
                    line.consume_line(&mut self.errors);
                    return Ok(State::Incomplete(self));
                }
            },
        };

        // store the statement data for later parsing
        match state {
            Ok(statement::State::Complete(statement)) => self.body.push(statement),
            Ok(statement::State::Incomplete(parser)) => self.pending.push(parser),
            Err(mut errors) => self.errors.append(&mut errors),
        }

        // if we get here then the parsing is incomplete
        Ok(State::Incomplete(self))
    }
}
