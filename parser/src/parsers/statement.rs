use boba_script_core::ast::{
    func::Func, node::Builder, Expr, ExprNode, Node, Statement, StatementNode,
};

use crate::{
    error::PError,
    stream::{SourceExt, SourceSpan},
    ConsumeEnd, ConsumeFlag, ParseError, Token, TokenLine, TokenStream,
};

use super::{
    block::{self, BlockParser},
    expr, line,
};

pub enum StatementType<Source: SourceSpan> {
    SingleLine(StatementNode<Source>),
    MultiLine(StatementParser<Source>),
}

enum ParseKind<Source: SourceSpan> {
    While {
        source: Source,
        cond: ExprNode<Source>,
        block: BlockParser<Source>,
    },
    Func {
        source: Source,
        name: String,
        params: Vec<String>,
        block: BlockParser<Source>,
    },
}

pub struct StatementParser<Source: SourceSpan> {
    kind: Option<ParseKind<Source>>,
}

impl<Source: SourceSpan> StatementParser<Source> {
    pub fn is_none(&self) -> bool {
        self.kind.is_none()
    }

    pub fn none() -> Self {
        Self { kind: None }
    }

    pub fn parse_line<T: TokenStream<Source = Source>>(
        &mut self,
        line: &mut TokenLine<T>,
    ) -> Result<Option<StatementNode<Source>>, Vec<PError<T>>> {
        match self.kind.take() {
            None => Ok(None),
            Some(ParseKind::While {
                source,
                cond,
                mut block,
            }) => match block.parse_line(line) {
                Ok(Some(body)) => Ok(Some(Statement::While { cond, body }.build_node(source))),
                Ok(None) => {
                    self.kind = Some(ParseKind::While {
                        source,
                        cond,
                        block,
                    });
                    Ok(None)
                }
                Err(errors) => {
                    self.kind = Some(ParseKind::While {
                        source,
                        cond,
                        block,
                    });
                    Err(errors)
                }
            },
            Some(ParseKind::Func {
                source,
                name,
                params,
                mut block,
            }) => {
                let result = match block.parse_line(line) {
                    Ok(None) => Ok(None),
                    Err(errors) => Err(errors),
                    Ok(Some(body)) => {
                        let func = Func { params, body }.build_node(source.clone());
                        return Ok(Some(
                            Statement::Assign {
                                init: true,
                                lhs: Node::new(Expr::Var(name), source.clone()),
                                rhs: Node::new(Expr::Func(func), source.clone()),
                            }
                            .build_node(source),
                        ));
                    }
                };

                self.kind = Some(ParseKind::Func {
                    source,
                    name,
                    params,
                    block,
                });

                result
            }
        }
    }
}

pub fn start_parsing<T: TokenStream>(
    line: &mut TokenLine<T>,
) -> Result<StatementType<T::Source>, Vec<PError<T>>> {
    line.guard_else(
        |line| match line.peek_token() {
            // LET STATEMENTS
            Some(Ok(Token::Let)) => {
                // consume the let token
                line.consume_token();
                let start = line.token_start();

                // parse the lhs
                let lhs = expr::parse(line)?;

                // parse the assign symbol
                line.take_exact(Some(&Token::Assign)).map_err(|e| vec![e])?;

                // parse the rhs
                let rhs = expr::parse(line)?;

                // parse line close
                line::parse_close(line)?;

                // create source and build statement
                let source = line.build_source(start..rhs.source.end());
                Ok(StatementType::SingleLine(
                    Statement::Assign {
                        init: true,
                        lhs,
                        rhs,
                    }
                    .build_node(source),
                ))
            }

            // WHILE LOOP
            Some(Ok(Token::While)) => {
                // consume the while token
                line.consume_token();
                let start = line.token_start();

                // parse condition
                let cond = expr::parse(line)?;

                // build source for while header
                let source = line.build_source(start..cond.source.end());

                // parse the block header
                let block = block::start_parsing(line)?;

                // return the while parser
                Ok(StatementType::MultiLine(StatementParser {
                    kind: Some(ParseKind::While {
                        source,
                        cond,
                        block,
                    }),
                }))
            }

            Some(Ok(Token::If)) => {
                // consume the if token
                line.consume_token();
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

            Some(Ok(Token::Fn)) => {
                // consume the fn token
                line.consume_token();
                let start = line.token_start();

                // parse the function ident
                let name = match line.take_some("identifier").map_err(|e| vec![e])? {
                    Token::Ident(ident) => ident,
                    token => {
                        return Err(vec![ParseError::UnexpectedInput {
                            expect: "identifier".into(),
                            found: Some(token),
                            source: line.token_source(),
                        }])
                    }
                };

                // parse the open paren
                line.take_exact(Some(&Token::OpenParen))
                    .map_err(|e| vec![e])?;

                // parse the parameters
                let mut params = Vec::new();
                let end = line.guard_else(
                    |line| loop {
                        // parse closing paren or ident
                        match line.take_some("identifier or ')'").map_err(|e| vec![e])? {
                            Token::CloseParen => break Ok(line.token_end()),
                            Token::Ident(ident) => params.push(ident),
                            token => {
                                return Err(vec![ParseError::UnexpectedInput {
                                    expect: "identifier or ')'".into(),
                                    found: Some(token),
                                    source: line.token_source(),
                                }])
                            }
                        }

                        // parse comma or closing paren
                        match line.take_some("',' or ')'").map_err(|e| vec![e])? {
                            Token::Comma => continue,
                            Token::CloseParen => break Ok(line.token_end()),
                            token => {
                                break Err(vec![ParseError::UnexpectedInput {
                                    expect: "',' or ')'".into(),
                                    found: Some(token),
                                    source: line.token_source(),
                                }])
                            }
                        }
                    },
                    |errors| {
                        // consume until the end of braces
                        match errors.consume_until(|t| match t {
                            Token::CloseParen => ConsumeFlag::Inclusive,
                            _ => ConsumeFlag::Ignore,
                        }) {
                            // if the error found a closing paren, then finish
                            ConsumeEnd::Inclusive(_) => {}
                            // otherwise, push an unclosed brace error too
                            _ => errors.push(ParseError::UnclosedBrace {
                                open: errors.line().build_source(start..start + 1),
                                end: errors.line().token_end_source(),
                            }),
                        }
                    },
                )?;

                // build source for function header
                let source = line.build_source(start..end);

                // parse the block header
                let block = block::start_parsing(line)?;

                // return the function parser
                Ok(StatementType::MultiLine(StatementParser {
                    kind: Some(ParseKind::Func {
                        source,
                        name,
                        params,
                        block,
                    }),
                }))
            }

            // ASSIGNMENT OR EXPRESSION
            Some(_) => {
                // parse initial expression
                let expr = expr::parse(line)?;

                // parse into either an assignment or expression
                line.take_guard(|token, line| match token {
                    // OPEN EXPRESSION
                    Some(Token::Newline) | None => {
                        // create source and build open expression
                        let source = line.build_source(expr.source.span());
                        Ok(StatementType::SingleLine(
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
                        line.take_exact(None).map_err(|e| vec![e])?;

                        // create source and build closed expression
                        let source = line.build_source(expr.source.span());
                        Ok(StatementType::SingleLine(
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
                        Ok(StatementType::SingleLine(
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
                source: line.token_source(),
            }]),
        },
        |errors| {
            // if an error is found, just consume the line
            errors.consume_line();
        },
    )
}
