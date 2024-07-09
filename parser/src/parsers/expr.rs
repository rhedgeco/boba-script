use boba_script_core::ast::{node::Builder, Expr, ExprNode};

use crate::{
    error::SpanParseError,
    stream::{Source, SourceSpan, StreamParser, StreamSpan},
    ParseError, Token, TokenStream,
};

pub fn parse<T: TokenStream>(
    parser: &mut StreamParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<ParseError<T>>> {
    let lhs = parse_atom(parser)?;
    parse_with_lhs(lhs, parser)
}

pub fn parse_atom<T: TokenStream>(
    parser: &mut StreamParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<ParseError<T>>> {
    match parser.next_some("expression").map_err(|e| vec![e])? {
        // VALUES
        Token::None => Ok(Expr::None.build_node(parser.token_span())),
        Token::Bool(value) => Ok(Expr::Bool(value).build_node(parser.token_span())),
        Token::Int(value) => Ok(Expr::Int(value).build_node(parser.token_span())),
        Token::Float(value) => Ok(Expr::Float(value).build_node(parser.token_span())),
        Token::String(value) => Ok(Expr::String(value).build_node(parser.token_span())),

        // VARS / CALLS
        Token::Ident(ident) => Ok(Expr::Var(ident).build_node(parser.token_span())),

        // PARENTHESIS
        Token::OpenParen => {
            let open = parser.token_span();

            // match in a loop so that tuples can be parsed
            let mut exprs = Vec::new();
            let expr = loop {
                let expr = match parse(parser) {
                    Ok(expr) => expr,
                    Err(mut errors) => {
                        // consume the rest of the expression
                        parser.consume_until_with(&mut errors, |t| {
                            matches!(t, Token::Newline | Token::CloseParen)
                        });

                        // check for closing paren
                        match parser.peek() {
                            Some(Ok(Token::CloseParen)) => {
                                parser.next(); // consume closing paren
                            }
                            Some(_) | None => errors.push(SpanParseError::UnclosedBrace {
                                open,
                                end: parser.token_span_end(),
                            }),
                        };

                        return Err(errors);
                    }
                };

                // check for comma
                match parser.peek_some("',' or ')'") {
                    Ok(Token::Comma) => {
                        parser.next(); // consume comma
                        exprs.push(expr);
                    }
                    Ok(_) => break expr,
                    Err(error) => {
                        let mut errors = vec![error];
                        parser.consume_until_with(&mut errors, |t| {
                            matches!(t, Token::Newline | Token::CloseParen)
                        });
                        return Err(errors);
                    }
                }
            };

            // check for closing paren
            if let Err(error) = parser.next_expect(Some(&Token::CloseParen)) {
                // consume the rest of the expression
                let mut errors = vec![error];
                parser.consume_until_with(&mut errors, |t| {
                    matches!(t, Token::Newline | Token::CloseParen)
                });

                // check for closing paren
                match parser.peek() {
                    Some(Ok(Token::CloseParen)) => {
                        parser.next(); // consume closing paren
                    }
                    Some(_) | None => errors.push(SpanParseError::UnclosedBrace {
                        open,
                        end: parser.token_span_end(),
                    }),
                };

                return Err(errors);
            }

            match exprs.is_empty() {
                true => Ok(expr),
                false => {
                    exprs.push(expr);
                    let span = parser.source().span(open.start()..parser.token_end());
                    Ok(Expr::Tuple(exprs).build_node(span))
                }
            }
        }

        // UNEXPECTED TOKEN
        token => Err(vec![SpanParseError::UnexpectedInput {
            expect: "expression".into(),
            found: Some(token),
            span: parser.token_span(),
        }]),
    }
}

pub fn parse_with_lhs<T: TokenStream>(
    mut lhs: ExprNode<StreamSpan<T>>,
    parser: &mut StreamParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<ParseError<T>>> {
    // keep parsing operators until an invalid operator is found
    loop {
        lhs = match parser.peek() {
            Some(Err(error)) => return Err(vec![error]),
            Some(Ok(token)) => match token {
                Token::Pow => parse_pow(lhs, parser)?,
                Token::Mul | Token::Div | Token::Modulo => parse_mul(lhs, parser)?,
                Token::Add | Token::Sub => parse_add(lhs, parser)?,
                Token::Eq | Token::Lt | Token::Gt | Token::NEq | Token::LtEq | Token::GtEq => {
                    parse_relation(lhs, parser)?
                }
                Token::And => parse_and(lhs, parser)?,
                Token::Or => parse_or(lhs, parser)?,
                Token::Question => parse_ternary(lhs, parser)?,
                Token::Walrus => parse_walrus(lhs, parser)?,
                _ => return Ok(lhs),
            },
            _ => return Ok(lhs),
        }
    }
}

pub fn parse_pow<T: TokenStream>(
    lhs: ExprNode<StreamSpan<T>>,
    parser: &mut StreamParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<ParseError<T>>> {
    let op = match parser.peek() {
        Some(Ok(Token::Pow)) => Expr::Pow,
        Some(Ok(_)) | None => return Ok(lhs),
        Some(Err(error)) => return Err(vec![error]),
    };

    parser.next(); // consume op
    let rhs = parse_atom(parser)?;
    let rhs = parse_pow(rhs, parser)?; // parse right to left
    let span = parser.source().span(lhs.data.start()..rhs.data.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(span))
}

pub fn parse_mul<T: TokenStream>(
    lhs: ExprNode<StreamSpan<T>>,
    parser: &mut StreamParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<ParseError<T>>> {
    let op = match parser.peek() {
        Some(Ok(Token::Mul)) => Expr::Mul,
        Some(Ok(Token::Div)) => Expr::Div,
        Some(Ok(Token::Modulo)) => Expr::Modulo,
        Some(Ok(_)) | None => return parse_pow(lhs, parser), // try next level up
        Some(Err(error)) => return Err(vec![error]),
    };

    parser.next(); // consume op
    let rhs = parse_atom(parser)?;
    let rhs = parse_pow(rhs, parser)?; // parse higher precedence
    let span = parser.source().span(lhs.data.start()..rhs.data.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(span))
}

pub fn parse_add<T: TokenStream>(
    lhs: ExprNode<StreamSpan<T>>,
    parser: &mut StreamParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<ParseError<T>>> {
    let op = match parser.peek() {
        Some(Ok(Token::Add)) => Expr::Add,
        Some(Ok(Token::Sub)) => Expr::Sub,
        Some(Ok(_)) | None => return parse_mul(lhs, parser), // try next level up
        Some(Err(error)) => return Err(vec![error]),
    };

    parser.next(); // consume op
    let rhs = parse_atom(parser)?;
    let rhs = parse_mul(rhs, parser)?; // parse higher precedence
    let span = parser.source().span(lhs.data.start()..rhs.data.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(span))
}

pub fn parse_relation<T: TokenStream>(
    lhs: ExprNode<StreamSpan<T>>,
    parser: &mut StreamParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<ParseError<T>>> {
    let op = match parser.peek() {
        Some(Ok(Token::Eq)) => Expr::Eq,
        Some(Ok(Token::Lt)) => Expr::Lt,
        Some(Ok(Token::Gt)) => Expr::Gt,
        Some(Ok(Token::NEq)) => Expr::NEq,
        Some(Ok(Token::LtEq)) => Expr::LtEq,
        Some(Ok(Token::GtEq)) => Expr::GtEq,
        Some(Ok(_)) | None => return parse_add(lhs, parser), // try next level up
        Some(Err(error)) => return Err(vec![error]),
    };

    parser.next(); // consume op
    let rhs = parse_atom(parser)?;
    let rhs = parse_add(rhs, parser)?; // parse higher precedence
    let span = parser.source().span(lhs.data.start()..rhs.data.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(span))
}

pub fn parse_and<T: TokenStream>(
    lhs: ExprNode<StreamSpan<T>>,
    parser: &mut StreamParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<ParseError<T>>> {
    let op = match parser.peek() {
        Some(Ok(Token::And)) => Expr::And,
        Some(Ok(_)) | None => return parse_relation(lhs, parser), // try next level up
        Some(Err(error)) => return Err(vec![error]),
    };

    parser.next(); // consume op
    let rhs = parse_atom(parser)?;
    let rhs = parse_relation(rhs, parser)?; // parse higher precedence
    let span = parser.source().span(lhs.data.start()..rhs.data.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(span))
}

pub fn parse_or<T: TokenStream>(
    lhs: ExprNode<StreamSpan<T>>,
    parser: &mut StreamParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<ParseError<T>>> {
    let op = match parser.peek() {
        Some(Ok(Token::Or)) => Expr::Or,
        Some(Ok(_)) | None => return parse_and(lhs, parser), // try next level up
        Some(Err(error)) => return Err(vec![error]),
    };

    parser.next(); // consume op
    let rhs = parse_atom(parser)?;
    let rhs = parse_and(rhs, parser)?; // parse higher precedence
    let span = parser.source().span(lhs.data.start()..rhs.data.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(span))
}

pub fn parse_ternary<T: TokenStream>(
    cond: ExprNode<StreamSpan<T>>,
    parser: &mut StreamParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<ParseError<T>>> {
    // parse the question mark
    match parser.peek() {
        Some(Ok(Token::Question)) => (),
        _ => return parse_or(cond, parser), // try next level up
    };
    parser.next(); // consume question

    // parse the pass condition
    let lhs = parse_atom(parser)?;
    let pass = parse_or(lhs, parser)?; // parse higher precedence

    // parse the colon seperator
    parser
        .next_expect(Some(&Token::Colon))
        .map_err(|e| vec![e])?;

    // parse the fail condition
    let lhs = parse_atom(parser)?;
    let fail = parse_or(lhs, parser)?; // parse higher precedence

    // build the ternary
    let span = parser.source().span(cond.data.start()..fail.data.end());
    Ok(Expr::Ternary {
        cond: Box::new(cond),
        pass: Box::new(pass),
        fail: Box::new(fail),
    }
    .build_node(span))
}

pub fn parse_walrus<T: TokenStream>(
    lhs: ExprNode<StreamSpan<T>>,
    parser: &mut StreamParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<ParseError<T>>> {
    let op = match parser.peek() {
        Some(Ok(Token::Walrus)) => Expr::Walrus,
        Some(Ok(_)) | None => return parse_ternary(lhs, parser), // try next level up
        Some(Err(error)) => return Err(vec![error]),
    };

    parser.next(); // consume op
    let rhs = parse_atom(parser)?;
    let rhs = parse_ternary(rhs, parser)?; // parse higher precedence
    let span = parser.source().span(lhs.data.start()..rhs.data.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(span))
}
