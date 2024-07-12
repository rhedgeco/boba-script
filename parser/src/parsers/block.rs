use boba_script_core::ast::StatementNode;

use crate::{error::PError, stream::SourceSpan, ParseError, Token, TokenLine, TokenStream};

use super::statement::{self, StatementParser};

pub fn start_parsing<T: TokenStream>(
    line: &mut TokenLine<T>,
) -> Result<State<T::Source, T::Error>, Vec<PError<T>>> {
    line.take_guard_else(
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
                line.take_exact(None).map_err(|e| vec![e])?;

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

pub enum State<Source: SourceSpan, Error> {
    Complete(Vec<StatementNode<Source>>),
    Incomplete(BlockParser<Source, Error>),
}

pub struct BlockParser<Source: SourceSpan, Error> {
    pending: Vec<StatementParser<Source, Error>>,
    errors: Vec<ParseError<Source, Error>>,
    body: Vec<StatementNode<Source>>,
    source: Source,
}

impl<Source: SourceSpan, Error> BlockParser<Source, Error> {
    pub fn source(&self) -> Source {
        self.source.clone()
    }

    pub fn parse_line<T: TokenStream<Source = Source, Error = Error>>(
        mut self,
        line: &mut TokenLine<T>,
    ) -> Result<State<Source, Error>, Vec<PError<T>>> {
        // if the body is empty, ensure that it starts with an indent token
        if self.body.is_empty() {
            match line.peek_token() {
                // consume indent if found
                Some(Ok(Token::Indent)) => line.consume_token(),

                // otherwise produce an empty body error
                _ => {
                    return Err(vec![ParseError::EmptyBlock {
                        source: self.source.clone(),
                    }])
                }
            }
        }

        // parse any pending statements
        let state = match self.pending.pop() {
            Some(parser) => parser.parse_line(line),

            // if no more statements are pending check for dedent
            None => match line.peek_token() {
                // if we find a dedent, then end parsing and return the data
                Some(Ok(Token::Dedent)) => match self.errors.is_empty() {
                    true => return Ok(State::Complete(self.body)),
                    false => return Err(self.errors),
                },

                // if we find anything else, parse the line as a statement
                _ => statement::start_parsing(line),
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
