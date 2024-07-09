use boba_script_core::ast::{statement, Statement};

use crate::{
    error::SpanParseError,
    parsers::expr,
    stream::{Source, SourceSpan, StreamSpan},
    ParseError, StreamParser, Token, TokenStream,
};

pub fn parse<T: TokenStream>(
    parser: &mut StreamParser<T>,
) -> Result<Statement<StreamSpan<T>>, Vec<ParseError<T>>> {
    match parser.peek_some("let or expression") {
        // consume the whole line if an error is found
        Err(error) => {
            let mut errors = vec![error];
            parser.consume_until_with(&mut errors, |t| {
                matches!(t, Token::SemiColon | Token::Newline)
            });
            Err(errors)
        }
        Ok(Token::Let) => {
            // get the start index of the let token
            parser.next(); // consume let token
            let start = parser.token_start();

            // parse the lhs
            let lhs = match expr::parse(parser) {
                Ok(lhs) => lhs,
                Err(mut errors) => {
                    parser.consume_until_with(&mut errors, |t| {
                        matches!(t, Token::SemiColon | Token::Newline)
                    });
                    return Err(errors);
                }
            };

            // parse the assignment symbol
            if let Err(error) = parser.next_expect(Some(&Token::Assign)) {
                let mut errors = vec![error];
                parser.consume_until_with(&mut errors, |t| {
                    matches!(t, Token::SemiColon | Token::Newline)
                });
                return Err(errors);
            }

            // parse the rhs
            let rhs = match expr::parse(parser) {
                Ok(lhs) => lhs,
                Err(mut errors) => {
                    parser.consume_until_with(&mut errors, |t| {
                        matches!(t, Token::SemiColon | Token::Newline)
                    });
                    return Err(errors);
                }
            };

            // parse close for consitency
            parse_close_and_end(parser)?;

            let span = parser.source().span(start..rhs.data.end());
            Ok(statement::Kind::Assign {
                init: true,
                lhs,
                rhs,
            }
            .carry(span))
        }
        Ok(Token::While) => {
            // get the start index of the while token
            parser.next(); // consume while token
            let start = parser.token_start();

            // parse the condition
            let cond = match expr::parse(parser) {
                Ok(lhs) => lhs,
                Err(mut errors) => {
                    parser.consume_until_with(&mut errors, |t| {
                        matches!(t, Token::SemiColon | Token::Newline)
                    });
                    return Err(errors);
                }
            };

            let colon = match parser.next_some("':' or '=>'") {
                // a colon means that the while loop has a body
                Ok(Token::Colon) => parser.token_end(),
                // a fat arrow is immediately followed by a statement
                Ok(Token::FatArrow) => {
                    let body = parse(parser)?;
                    let span = parser.source().span(start..body.data.end());
                    return Ok(statement::Kind::While {
                        cond,
                        body: vec![body],
                    }
                    .carry(span));
                }
                Ok(token) => {
                    let mut errors = vec![SpanParseError::UnexpectedInput {
                        expect: "':' or ';".into(),
                        found: Some(token),
                        span: parser.token_span(),
                    }];
                    parser.consume_until_with(&mut errors, |t| {
                        matches!(t, Token::SemiColon | Token::Newline)
                    });
                    return Err(errors);
                }
                Err(error) => {
                    let mut errors = vec![error];
                    parser.consume_line_with(&mut errors);
                    return Err(errors);
                }
            };

            // check line end
            if let Err(error) = parser.next_line_end() {
                let mut errors = vec![error];
                parser.consume_line_with(&mut errors);
                return Err(errors);
            }

            // parse body statements
            let mut body = Vec::new();
            match parser.peek() {
                Some(Err(error)) => return Err(vec![error]),
                Some(Ok(Token::Indent)) => {
                    parser.next(); // consume indent
                    let mut errors = Vec::new();
                    loop {
                        // parse statement
                        match parse(parser) {
                            Err(mut parse_errors) => errors.append(&mut parse_errors),
                            Ok(statement) => body.push(statement),
                        }

                        // check for dedent
                        match parser.peek() {
                            Some(Ok(Token::Dedent)) => {
                                parser.next();
                                break;
                            }
                            Some(Err(error)) => {
                                let mut errors = vec![error];
                                parser.consume_until_with(&mut errors, |t| {
                                    matches!(t, Token::Dedent)
                                });
                                return Err(errors);
                            }
                            Some(_) | None => {}
                        }
                    }
                }
                Some(_) | None => {}
            }

            let span = match body.last() {
                Some(body) => parser.source().span(start..body.data.end()),
                None => parser.source().span(start..colon),
            };
            Ok(statement::Kind::While { cond, body }.carry(span))
        }
        Ok(_) => {
            // first parse the lhs
            let lhs = match expr::parse(parser) {
                Ok(lhs) => lhs,
                Err(mut errors) => {
                    parser.consume_until_with(&mut errors, |t| {
                        matches!(t, Token::SemiColon | Token::Newline)
                    });
                    return Err(errors);
                }
            };

            // then check for an assignment
            match parser.next() {
                Some(Ok(Token::Assign)) => (),
                Some(Ok(Token::Newline)) | None => {
                    let span = parser.source().span(lhs.data.span());
                    return Ok(statement::Kind::Expr {
                        expr: lhs,
                        closed: false,
                    }
                    .carry(span));
                }
                Some(Ok(Token::SemiColon)) => {
                    // ensure line ends after semicolon
                    match parser.next_line_end() {
                        Ok(_) => (),
                        Err(error) => {
                            let mut errors = vec![error];
                            parser.consume_line_with(&mut errors);
                            return Err(errors);
                        }
                    }

                    let span = parser.source().span(lhs.data.span());
                    return Ok(statement::Kind::Expr {
                        expr: lhs,
                        closed: true,
                    }
                    .carry(span));
                }
                Some(Ok(token)) => {
                    let mut errors = vec![SpanParseError::UnexpectedInput {
                        expect: "'=' or line end".into(),
                        found: Some(token),
                        span: parser.token_span(),
                    }];
                    parser.consume_until_with(&mut errors, |t| {
                        matches!(t, Token::SemiColon | Token::Newline)
                    });
                    return Err(errors);
                }
                Some(Err(error)) => {
                    let mut errors = vec![error];
                    parser.consume_until_with(&mut errors, |t| {
                        matches!(t, Token::SemiColon | Token::Newline)
                    });
                    return Err(errors);
                }
            }

            // if an assignment was found, then parse the rhs
            let rhs = match expr::parse(parser) {
                Ok(lhs) => lhs,
                Err(mut errors) => {
                    parser.consume_until_with(&mut errors, |t| {
                        matches!(t, Token::SemiColon | Token::Newline)
                    });
                    return Err(errors);
                }
            };

            // parse close for consitency
            parse_close_and_end(parser)?;

            let span = parser.source().span(lhs.data.start()..rhs.data.end());
            Ok(statement::Kind::Assign {
                init: false,
                lhs,
                rhs,
            }
            .carry(span))
        }
    }
}

fn parse_close_and_end<T: TokenStream>(
    parser: &mut StreamParser<T>,
) -> Result<bool, Vec<ParseError<T>>> {
    match parser.next() {
        Some(Ok(Token::Newline)) | None => Ok(false),
        Some(Err(error)) => {
            let mut errors = vec![error];
            parser.consume_until_with(&mut errors, |t| {
                matches!(t, Token::SemiColon | Token::Newline)
            });
            Err(errors)
        }
        Some(Ok(Token::SemiColon)) => {
            if let Err(error) = parser.next_line_end() {
                let mut errors = vec![error];
                parser.consume_line_with(&mut errors);
                return Err(errors);
            }
            Ok(true)
        }
        Some(Ok(token)) => {
            let mut errors = vec![SpanParseError::UnexpectedInput {
                expect: "';' or line end".into(),
                found: Some(token),
                span: parser.token_span(),
            }];
            parser.consume_until_with(&mut errors, |t| {
                matches!(t, Token::SemiColon | Token::Newline)
            });
            Err(errors)
        }
    }
}
