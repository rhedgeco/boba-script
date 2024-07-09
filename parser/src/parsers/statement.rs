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
    let statement = match parser.peek_some("let or expression") {
        // consume the whole line if an error is found
        Err(error) => {
            let mut errors = vec![error];
            parser.consume_until_with(&mut errors, |t| {
                matches!(t, Token::SemiColon | Token::Newline)
            });
            return Err(errors);
        }
        Ok(Token::Let) => {
            // get the start index of the let token
            parser.next(); // consume let token first
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

            let span = parser.source().span(start..rhs.data.end());
            statement::Kind::Assign {
                init: true,
                lhs,
                rhs,
            }
            .carry(span)
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

            let span = parser.source().span(lhs.data.start()..rhs.data.end());
            statement::Kind::Assign {
                init: false,
                lhs,
                rhs,
            }
            .carry(span)
        }
    };

    // check for semicolon or newline
    // allow for semicolons here just for consistency
    // the semicolons have no effect on the statement at this point
    match parser.next() {
        Some(Ok(Token::Newline)) | None => (),
        Some(Err(error)) => {
            let mut errors = vec![error];
            parser.consume_until_with(&mut errors, |t| {
                matches!(t, Token::SemiColon | Token::Newline)
            });
            return Err(errors);
        }
        Some(Ok(Token::SemiColon)) => {
            if let Err(error) = parser.next_line_end() {
                let mut errors = vec![error];
                parser.consume_line_with(&mut errors);
                return Err(errors);
            }
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
            return Err(errors);
        }
    }

    Ok(statement)
}
