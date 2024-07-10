use boba_script_core::ast::{node::Builder, ExprNode, Statement, StatementNode};

use crate::{
    error::ParseError,
    parsers::expr,
    stream::{LineParser, Source, TokenLine},
    PError, Token, TokenStream,
};

use super::{block, line};

pub enum Header<Data: Source> {
    Complete(StatementNode<Data>),
    Incomplete(IncompleteStatement<Data>),
}

enum IncompleteKind<Data> {
    While(ExprNode<Data>),
    If(ExprNode<Data>),
}

impl<Data> IncompleteKind<Data> {
    fn build(self, source: Data, body: Vec<StatementNode<Data>>) -> StatementNode<Data> {
        match self {
            IncompleteKind::While(cond) => Statement::While { cond, body }.build_node(source),
            IncompleteKind::If(cond) => Statement::If {
                cond,
                pass: body,
                fail: vec![],
            }
            .build_node(source),
        }
    }
}

pub struct IncompleteStatement<Data: Source> {
    kind: IncompleteKind<Data>,
    block_source: Data,
    source: Data,
}

impl<Data: Source> IncompleteStatement<Data> {
    pub fn finish<T: TokenStream<Source = Data>>(
        self,
        parser: &mut LineParser<T>,
    ) -> Result<StatementNode<T::Source>, Vec<PError<T>>> {
        let header = block::Header::Incomplete(self.block_source);
        let body = block::parse_with_header(header, parser)?;
        Ok(self.kind.build(self.source, body))
    }

    pub fn finish_with<E>(
        self,
        body: Vec<StatementNode<Data>>,
    ) -> Result<StatementNode<Data>, ParseError<Data, E>> {
        match body.is_empty() {
            false => Ok(self.kind.build(self.source, body)),
            true => Err(ParseError::EmptyBlock {
                source: self.block_source,
            }),
        }
    }
}

pub fn parse<T: TokenStream>(
    parser: &mut LineParser<T>,
) -> Result<StatementNode<T::Source>, Vec<PError<T>>> {
    match parse_header(&mut parser.line())? {
        Header::Complete(statement) => Ok(statement),
        Header::Incomplete(statement) => statement.finish(parser),
    }
}

pub fn parse_inline<T: TokenStream>(
    line: &mut TokenLine<T>,
) -> Result<StatementNode<T::Source>, Vec<PError<T>>> {
    match parse_header(line)? {
        // COMPLETE STATEMENT
        Header::Complete(statement) => Ok(statement),

        // FAILURE CASE
        Header::Incomplete(incomplete) => Err(vec![ParseError::InlineError {
            source: incomplete.block_source,
        }]),
    }
}

pub fn parse_header<T: TokenStream>(
    line: &mut TokenLine<T>,
) -> Result<Header<T::Source>, Vec<PError<T>>> {
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
                let source = line.build_source(start..rhs.data.end());
                Ok(Header::Complete(
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
                let source = line.build_source(start..cond.data.end());

                // parse the block header
                match block::parse_header(line)? {
                    block::Header::Complete(statement) => Ok(Header::Complete(
                        Statement::While {
                            cond,
                            body: vec![statement],
                        }
                        .build_node(source),
                    )),
                    block::Header::Incomplete(block_source) => {
                        Ok(Header::Incomplete(IncompleteStatement {
                            kind: IncompleteKind::While(cond),
                            block_source,
                            source,
                        }))
                    }
                }
            }

            Some(Token::If) => {
                // consume the if token
                let line = &mut peeker.consume();
                let start = line.token_start();

                // parse condition
                let cond = expr::parse(line)?;

                // build source for if header
                let source = line.build_source(start..cond.data.end());

                // parse the block header
                match block::parse_header(line)? {
                    block::Header::Complete(statement) => Ok(Header::Complete(
                        Statement::If {
                            cond,
                            pass: vec![statement],
                            fail: vec![],
                        }
                        .build_node(source),
                    )),
                    block::Header::Incomplete(block_source) => {
                        Ok(Header::Incomplete(IncompleteStatement {
                            kind: IncompleteKind::If(cond),
                            block_source,
                            source,
                        }))
                    }
                }
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
                        let source = line.build_source(expr.data.span());
                        Ok(Header::Complete(
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
                        let source = line.build_source(expr.data.span());
                        Ok(Header::Complete(
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
                        let source = line.build_source(expr.data.start()..rhs.data.end());
                        Ok(Header::Complete(
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
