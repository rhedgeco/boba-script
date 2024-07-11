use boba_script_core::ast::{node::Builder, ExprNode, Statement, StatementNode};

use crate::{
    error::ParseError,
    parsers::expr,
    stream::{SpanSource, TokenLine},
    PError, Token, TokenStream,
};

use super::{
    block::{self, BlockParser},
    line,
};

pub enum State<Source: SpanSource, Error> {
    Complete(StatementNode<Source>),
    Incomplete(StatementParser<Source, Error>),
}

enum ParserKind<Source: SpanSource> {
    While {
        source: Source,
        cond: ExprNode<Source>,
    },
}

pub struct StatementParser<Source: SpanSource, Error> {
    block: BlockParser<Source, Error>,
    kind: ParserKind<Source>,
}

impl<Source: SpanSource, Error> StatementParser<Source, Error> {
    pub fn parse_line<T: TokenStream<Source = Source, Error = Error>>(
        mut self,
        line: &mut TokenLine<T>,
    ) -> Result<State<Source, Error>, Vec<PError<T>>> {
        let body = match self.block.parse_line(line)? {
            block::State::Complete(body) => body,
            block::State::Incomplete(block) => {
                self.block = block;
                return Ok(State::Incomplete(self));
            }
        };

        match self.kind {
            ParserKind::While { source, cond } => Ok(State::Complete(
                Statement::While { cond, body }.build_node(source),
            )),
        }
    }
}

pub fn parse_inline<T: TokenStream>(
    inline_source: T::Source,
    line: &mut TokenLine<T>,
) -> Result<StatementNode<T::Source>, Vec<PError<T>>> {
    match start_parsing(line)? {
        // COMPLETE STATEMENT
        State::Complete(statement) => Ok(statement),

        // FAILURE CASE
        State::Incomplete(parser) => Err(vec![ParseError::InlineError {
            block_source: parser.block.source(),
            inline_source: inline_source,
        }]),
    }
}

pub fn start_parsing<T: TokenStream>(
    line: &mut TokenLine<T>,
) -> Result<State<T::Source, T::Error>, Vec<PError<T>>> {
    line.parse_peek_else(
        |peeker| match peeker.token() {
            // LET STATEMENTS
            Some(Token::Let) => {
                // consume the let token
                let line = &mut peeker.consume();
                let start = line.token_start();

                // parse the lhs
                let lhs = expr::parse(line)?;

                // parse the assign symbol
                line.take_expect(Some(&Token::Assign))
                    .map_err(|e| vec![e])?;

                // parse the rhs
                let rhs = expr::parse(line)?;

                // parse line close
                line::parse_close(line)?;

                // create source and build statement
                let source = line.build_source(start..rhs.source.end());
                Ok(State::Complete(
                    Statement::Assign {
                        init: true,
                        lhs,
                        rhs,
                    }
                    .build_node(source),
                ))
            }

            // WHILE LOOP
            Some(Token::While) => {
                // consume the while token
                let line = &mut peeker.consume();
                let start = line.token_start();

                // parse condition
                let cond = expr::parse(line)?;

                // build source for while header
                let source = line.build_source(start..cond.source.end());

                // parse the block header
                match block::start_parsing(line)? {
                    block::State::Complete(body) => Ok(State::Complete(
                        Statement::While { cond, body }.build_node(source),
                    )),
                    block::State::Incomplete(block) => Ok(State::Incomplete(StatementParser {
                        kind: ParserKind::While { source, cond },
                        block,
                    })),
                }
            }

            Some(Token::If) => {
                // consume the if token
                let line = &mut peeker.consume();
                let start = line.token_start();

                // parse condition
                let cond = expr::parse(line)?;

                // build source for if header
                let _source = line.build_source(start..cond.source.end());

                // parse the block header
                // match block::parse_header(line)? {
                //     block::Header::Complete(statement) => Ok(State::Complete(
                //         Statement::If {
                //             cond,
                //             pass: vec![statement],
                //             fail: vec![],
                //         }
                //         .build_node(source),
                //     )),
                //     block::Header::Incomplete(block_source) => Ok(State::Block(BlockStatement {
                //         kind: BlockKind::If(cond),
                //         block_source,
                //         source,
                //     })),
                // }

                todo!()
            }

            // ASSIGNMENT OR EXPRESSION
            Some(_) => {
                // ignore the peeked token
                let line = &mut peeker.ignore();

                // parse initial expression
                let expr = expr::parse(line)?;

                // parse into either an assignment or expression
                line.parse_next(|token, line| match token {
                    // OPEN EXPRESSION
                    Some(Token::Newline) | None => {
                        // create source and build open expression
                        let source = line.build_source(expr.source.span());
                        Ok(State::Complete(
                            Statement::Expr {
                                expr,
                                closed: false,
                            }
                            .build_node(source),
                        ))
                    }

                    // CLOSED EXPRESSION
                    Some(Token::SemiColon) => {
                        // parse line end
                        line.take_expect(None).map_err(|e| vec![e])?;

                        // create source and build closed expression
                        let source = line.build_source(expr.source.span());
                        Ok(State::Complete(
                            Statement::Expr { expr, closed: true }.build_node(source),
                        ))
                    }

                    // ASSIGNMENT
                    Some(Token::Assign) => {
                        // parse rhs expression
                        let rhs = expr::parse(line)?;

                        // parse line close
                        line::parse_close(line)?;

                        // create source and build assignment
                        let source = line.build_source(expr.source.start()..rhs.source.end());
                        Ok(State::Complete(
                            Statement::Assign {
                                init: false,
                                lhs: expr,
                                rhs,
                            }
                            .build_node(source),
                        ))
                    }

                    // FAILURE CASE
                    token => Err(vec![ParseError::UnexpectedInput {
                        expect: "'=', ';', or end of line".into(),
                        found: token,
                        source: line.token_source(),
                    }]),
                })
            }

            // FAILURE CASE
            None => Err(vec![ParseError::UnexpectedInput {
                expect: "'let' or expression".into(),
                found: None,
                source: peeker.token_source(),
            }]),
        },
        |errors| {
            // if an error is found, just consume until the end of the line
            errors.consume_until_inclusive(|t| matches!(t, Token::Newline));
        },
    )
}
