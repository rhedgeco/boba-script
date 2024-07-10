use boba_script_core::ast::{node::Builder, Statement, StatementNode};

use crate::{
    error::ParseError,
    parsers::{expr, line, statement},
    stream::{SourceSpan, StreamSpan},
    PError, Token, TokenParser, TokenStream,
};

pub fn parse<T: TokenStream>(
    parser: &mut TokenParser<T>,
) -> Result<StatementNode<StreamSpan<T>>, Vec<PError<T>>> {
    parser.parse_peek_else(
        |peeker| match peeker.token() {
            // LET STATEMENTS
            Some(Token::Let) => {
                // consume the let token
                let parser = peeker.consume();
                let start = parser.token_start();

                // parse the lhs
                let lhs = expr::parse(parser)?;

                // parse the assign symbol
                parser
                    .take_expect(Some(&Token::Assign))
                    .map_err(|e| vec![e])?;

                // parse the rhs
                let rhs = expr::parse(parser)?;

                // parse line close
                line::parse_close(parser)?;

                // create span and build statement
                let span = parser.span(start..rhs.data.end());
                Ok(Statement::Assign {
                    init: true,
                    lhs,
                    rhs,
                }
                .build_node(span))
            }

            // WHILE LOOP
            Some(Token::While) => {
                // consume the let token
                let parser = peeker.consume();
                let start = parser.token_start();

                // parse condition
                let cond = expr::parse(parser)?;

                // parse into either inline or multi line loop
                parser.parse_next(|token, parser| match token {
                    // INLINE WHILE LOOP
                    Some(Token::FatArrow) => {
                        // parse the body statement
                        let body = statement::parse(parser)?;

                        // create span and build statement
                        let span = parser.span(start..body.data.end());
                        Ok(Statement::While {
                            cond,
                            body: vec![body],
                        }
                        .build_node(span))
                    }

                    // MULTI LINE WHILE LOOP
                    Some(Token::Colon) => todo!("parse multi line while loop"),

                    // FAILURE CASE
                    token => Err(vec![ParseError::UnexpectedInput {
                        expect: "':', or '=>'".into(),
                        found: token,
                        span: parser.token_span(),
                    }]),
                })
            }

            // ASSIGNMENT OR EXPRESSION
            Some(_) => {
                // ignore the peeked token
                let parser = peeker.ignore();

                // parse the left expression
                let expr = expr::parse(parser)?;

                // parse into either an assignment or expression
                parser.parse_next(|token, parser| match token {
                    // ASSIGNMENT
                    Some(Token::Assign) => todo!(),

                    // OPEN EXPRESSION
                    Some(Token::Newline) | None => {
                        // create span and build open expression
                        let span = parser.span(expr.data.span());
                        Ok(Statement::Expr {
                            expr,
                            closed: false,
                        }
                        .build_node(span))
                    }

                    // CLOSED EXPRESSION
                    Some(Token::SemiColon) => {
                        // parse line end
                        line::parse_end(parser)?;

                        // create span and build closed expression
                        let span = parser.span(expr.data.span());
                        Ok(Statement::Expr { expr, closed: true }.build_node(span))
                    }

                    // FAILURE CASE
                    token => Err(vec![ParseError::UnexpectedInput {
                        expect: "'=', ';', or end of line".into(),
                        found: token,
                        span: parser.token_span(),
                    }]),
                })
            }

            // FAILURE CASE
            None => Err(vec![ParseError::UnexpectedInput {
                expect: "'let' or expression".into(),
                found: None,
                span: peeker.token_span(),
            }]),
        },
        |errors| {
            // if an error is found, just consume until the end of the line
            errors.consume_until(|t| matches!(t, Token::SemiColon | Token::Newline));
        },
    )

    // match parser.peek_some("let or expression") {
    //     // consume the whole line if an error is found
    //     Err(error) => {
    //         let mut errors = vec![error];
    //         parser.consume_until_with(&mut errors, |t| {
    //             matches!(t, Token::SemiColon | Token::Newline)
    //         });
    //         Err(errors)
    //     }
    //     Ok(Token::Let) => {
    //         // get the start index of the let token
    //         parser.next();
    //         let start = parser.token_start();

    //         // parse the lhs
    //         let lhs = expr::parse(parser).map_err(|mut errors| {
    //             parser.consume_until_with(&mut errors, |t| {
    //                 matches!(t, Token::SemiColon | Token::Newline)
    //             });
    //             errors
    //         })?;

    //         // parse the assignment symbol
    //         parser.parse_next(
    //             |token, parser| match token {
    //                 Some(Token::Assign) => Ok(()),
    //                 token => Err(vec![ParseError::UnexpectedInput {
    //                     expect: format!("'='"),
    //                     found: token,
    //                     span: parser.token_span(),
    //                 }]),
    //             },
    //             |parser| {
    //                 parser.consume_until(|t| matches!(t, Token::SemiColon | Token::Newline));
    //             },
    //         )?;

    //         // parse the rhs
    //         let rhs = expr::parse(parser).map_err(|mut errors| {
    //             parser.consume_until_with(&mut errors, |t| {
    //                 matches!(t, Token::SemiColon | Token::Newline)
    //             });
    //             errors
    //         })?;

    //         // parse close for consitency
    //         line::parse_close(parser)?;

    //         // build statement
    //         let span = parser.source().span(start..rhs.data.end());
    //         Ok(Statement::Assign {
    //             init: true,
    //             lhs,
    //             rhs,
    //         }
    //         .build_node(span))
    //     }
    //     Ok(Token::While) => {
    //         // get the start index of the while token
    //         parser.next(); // consume while token
    //         let start = parser.token_start();

    //         // parse the condition
    //         let cond = match expr::parse(parser) {
    //             Ok(lhs) => lhs,
    //             Err(mut errors) => {
    //                 parser.consume_until_with(&mut errors, |t| {
    //                     matches!(t, Token::SemiColon | Token::Newline)
    //                 });
    //                 return Err(errors);
    //             }
    //         };

    //         let colon = match parser.next_some("':' or '=>'") {
    //             // a colon means that the while loop has a body
    //             Ok(Token::Colon) => parser.token_end(),
    //             // a fat arrow is immediately followed by a statement
    //             Ok(Token::FatArrow) => {
    //                 let body = parse(parser)?;
    //                 let span = parser.source().span(start..body.data.end());
    //                 return Ok(Statement::While {
    //                     cond,
    //                     body: vec![body],
    //                 }
    //                 .build_node(span));
    //             }
    //             Ok(token) => {
    //                 let mut errors = vec![ParseError::UnexpectedInput {
    //                     expect: "':' or ';".into(),
    //                     found: Some(token),
    //                     span: parser.token_span(),
    //                 }];
    //                 parser.consume_until_with(&mut errors, |t| {
    //                     matches!(t, Token::SemiColon | Token::Newline)
    //                 });
    //                 return Err(errors);
    //             }
    //             Err(error) => {
    //                 let mut errors = vec![error];
    //                 parser.consume_line_with(&mut errors);
    //                 return Err(errors);
    //             }
    //         };

    //         // check line end
    //         if let Err(error) = parser.next_line_end() {
    //             let mut errors = vec![error];
    //             parser.consume_line_with(&mut errors);
    //             return Err(errors);
    //         }

    //         // parse body statements
    //         let mut body = Vec::new();
    //         match parser.peek() {
    //             Some(Err(error)) => return Err(vec![error]),
    //             Some(Ok(Token::Indent)) => {
    //                 parser.next(); // consume indent
    //                 let mut errors = Vec::new();
    //                 loop {
    //                     // parse statement
    //                     match parse(parser) {
    //                         Err(mut parse_errors) => errors.append(&mut parse_errors),
    //                         Ok(statement) => body.push(statement),
    //                     }

    //                     // check for dedent
    //                     match parser.peek() {
    //                         Some(Ok(Token::Dedent)) => {
    //                             parser.next();
    //                             break;
    //                         }
    //                         Some(Err(error)) => {
    //                             let mut errors = vec![error];
    //                             parser.consume_until_with(&mut errors, |t| {
    //                                 matches!(t, Token::Dedent)
    //                             });
    //                             return Err(errors);
    //                         }
    //                         Some(_) | None => {}
    //                     }
    //                 }
    //             }
    //             Some(_) | None => {}
    //         }

    //         let span = match body.last() {
    //             Some(body) => parser.source().span(start..body.data.end()),
    //             None => parser.source().span(start..colon),
    //         };
    //         Ok(Statement::While { cond, body }.build_node(span))
    //     }
    //     Ok(_) => {
    //         // first parse the lhs
    //         let lhs = match expr::parse(parser) {
    //             Ok(lhs) => lhs,
    //             Err(mut errors) => {
    //                 parser.consume_until_with(&mut errors, |t| {
    //                     matches!(t, Token::SemiColon | Token::Newline)
    //                 });
    //                 return Err(errors);
    //             }
    //         };

    //         // then check for an assignment
    //         match parser.next() {
    //             Some(Ok(Token::Assign)) => (),
    //             Some(Ok(Token::Newline)) | None => {
    //                 let span = parser.source().span(lhs.data.span());
    //                 return Ok(Statement::Expr {
    //                     expr: lhs,
    //                     closed: false,
    //                 }
    //                 .build_node(span));
    //             }
    //             Some(Ok(Token::SemiColon)) => {
    //                 // ensure line ends after semicolon
    //                 match parser.next_line_end() {
    //                     Ok(_) => (),
    //                     Err(error) => {
    //                         let mut errors = vec![error];
    //                         parser.consume_line_with(&mut errors);
    //                         return Err(errors);
    //                     }
    //                 }

    //                 let span = parser.source().span(lhs.data.span());
    //                 return Ok(Statement::Expr {
    //                     expr: lhs,
    //                     closed: true,
    //                 }
    //                 .build_node(span));
    //             }
    //             Some(Ok(token)) => {
    //                 let mut errors = vec![ParseError::UnexpectedInput {
    //                     expect: "'=' or line end".into(),
    //                     found: Some(token),
    //                     span: parser.token_span(),
    //                 }];
    //                 parser.consume_until_with(&mut errors, |t| {
    //                     matches!(t, Token::SemiColon | Token::Newline)
    //                 });
    //                 return Err(errors);
    //             }
    //             Some(Err(error)) => {
    //                 let mut errors = vec![error];
    //                 parser.consume_until_with(&mut errors, |t| {
    //                     matches!(t, Token::SemiColon | Token::Newline)
    //                 });
    //                 return Err(errors);
    //             }
    //         }

    //         // if an assignment was found, then parse the rhs
    //         let rhs = match expr::parse(parser) {
    //             Ok(lhs) => lhs,
    //             Err(mut errors) => {
    //                 parser.consume_until_with(&mut errors, |t| {
    //                     matches!(t, Token::SemiColon | Token::Newline)
    //                 });
    //                 return Err(errors);
    //             }
    //         };

    //         // parse close for consitency
    //         parse_close_and_end(parser)?;

    //         let span = parser.source().span(lhs.data.start()..rhs.data.end());
    //         Ok(Statement::Assign {
    //             init: false,
    //             lhs,
    //             rhs,
    //         }
    //         .build_node(span))
    //     }
    // }
}
