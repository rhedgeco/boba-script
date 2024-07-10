use boba_script_core::ast::{node::Builder, Expr, ExprNode};

use crate::{
    error::ParseError,
    stream::{SourceSpan, StreamSpan, TokenParser},
    PError, Token, TokenStream,
};

pub fn parse<T: TokenStream>(
    parser: &mut TokenParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<PError<T>>> {
    let lhs = parse_atom(parser)?;
    parse_with_lhs(lhs, parser)
}

pub fn parse_atom<T: TokenStream>(
    parser: &mut TokenParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<PError<T>>> {
    parser.parse_next(|token, parser| match token {
        // VALUES
        Some(Token::None) => Ok(Expr::None.build_node(parser.token_span())),
        Some(Token::Bool(value)) => Ok(Expr::Bool(value).build_node(parser.token_span())),
        Some(Token::Int(value)) => Ok(Expr::Int(value).build_node(parser.token_span())),
        Some(Token::Float(value)) => Ok(Expr::Float(value).build_node(parser.token_span())),
        Some(Token::String(value)) => Ok(Expr::String(value).build_node(parser.token_span())),

        // VARS / CALLS
        Some(Token::Ident(ident)) => Ok(Expr::Var(ident).build_node(parser.token_span())),

        // PARENTHESIS AND TUPLES
        Some(Token::OpenParen) => {
            // save the open paren span
            let start = parser.token_start();

            // parse all tuple parts if any
            let mut exprs = Vec::new();
            let expr = loop {
                // try parsing an inner expression
                let result = parser.parse_else(
                    |parser| {
                        // parse expression
                        let inner = parse(parser)?;

                        // then check for a comma or closing paren
                        let end = parser.parse_next(|token, parser| match token {
                            // a paren will tell the loop it is complete
                            Some(Token::CloseParen) => Ok(true),
                            // and a comma will tell the loop to continue
                            Some(Token::Comma) => Ok(false),
                            // otherwise it is an invalid token
                            token => Err(vec![ParseError::UnexpectedInput {
                                expect: "',' or ')'".into(),
                                found: token,
                                span: parser.token_span(),
                            }]),
                        })?;

                        // then return the inner expression
                        Ok((inner, end))
                    },
                    |errors| {
                        // consume until the end of the inner expression
                        match errors
                            .consume_until(|t| matches!(t, Token::CloseParen | Token::Newline))
                        {
                            // consume closing paren if found
                            Some(Token::CloseParen) => errors.consume_next(),
                            // if it was the end of the line, then also include an unclosed brace error
                            _ => errors.push(ParseError::UnclosedBrace {
                                open: errors.span(start..start + 1),
                                end: errors.token_end_span(),
                            }),
                        }
                    },
                );

                match result {
                    // immediately return any errors
                    Err(errors) => return Err(errors),
                    // or store tuple parameter
                    Ok((expr, false)) => exprs.push(expr),
                    // or break with the expression
                    Ok((expr, true)) => break expr,
                }
            };

            match exprs.is_empty() {
                // if there is only one expression
                // just return it as a normal expression
                true => Ok(expr),
                // otherwise combine the expressions to make a tuple
                false => {
                    exprs.push(expr);
                    let span = parser.span(start..parser.token_end());
                    Ok(Expr::Tuple(exprs).build_node(span))
                }
            }
        }

        // FAILURE CASE
        token => Err(vec![ParseError::UnexpectedInput {
            expect: "expression".into(),
            found: token,
            span: parser.token_span(),
        }]),
    })
}

pub fn parse_with_lhs<T: TokenStream>(
    mut lhs: ExprNode<StreamSpan<T>>,
    parser: &mut TokenParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<PError<T>>> {
    // keep parsing operators until an invalid operator is found
    loop {
        lhs = match parser.peek_next() {
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
    parser: &mut TokenParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<PError<T>>> {
    let op = match parser.peek_next() {
        Some(Ok(Token::Pow)) => Expr::Pow,
        Some(Ok(_)) | None => return Ok(lhs),
        Some(Err(error)) => return Err(vec![error]),
    };

    parser.take_next(); // consume op
    let rhs = parse_atom(parser)?;
    let rhs = parse_pow(rhs, parser)?; // parse right to left
    let span = parser.span(lhs.data.start()..rhs.data.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(span))
}

pub fn parse_mul<T: TokenStream>(
    lhs: ExprNode<StreamSpan<T>>,
    parser: &mut TokenParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<PError<T>>> {
    let op = match parser.peek_next() {
        Some(Ok(Token::Mul)) => Expr::Mul,
        Some(Ok(Token::Div)) => Expr::Div,
        Some(Ok(Token::Modulo)) => Expr::Modulo,
        Some(Ok(_)) | None => return parse_pow(lhs, parser), // try next level up
        Some(Err(error)) => return Err(vec![error]),
    };

    parser.take_next(); // consume op
    let rhs = parse_atom(parser)?;
    let rhs = parse_pow(rhs, parser)?; // parse higher precedence
    let span = parser.span(lhs.data.start()..rhs.data.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(span))
}

pub fn parse_add<T: TokenStream>(
    lhs: ExprNode<StreamSpan<T>>,
    parser: &mut TokenParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<PError<T>>> {
    let op = match parser.peek_next() {
        Some(Ok(Token::Add)) => Expr::Add,
        Some(Ok(Token::Sub)) => Expr::Sub,
        Some(Ok(_)) | None => return parse_mul(lhs, parser), // try next level up
        Some(Err(error)) => return Err(vec![error]),
    };

    parser.take_next(); // consume op
    let rhs = parse_atom(parser)?;
    let rhs = parse_mul(rhs, parser)?; // parse higher precedence
    let span = parser.span(lhs.data.start()..rhs.data.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(span))
}

pub fn parse_relation<T: TokenStream>(
    lhs: ExprNode<StreamSpan<T>>,
    parser: &mut TokenParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<PError<T>>> {
    let op = match parser.peek_next() {
        Some(Ok(Token::Eq)) => Expr::Eq,
        Some(Ok(Token::Lt)) => Expr::Lt,
        Some(Ok(Token::Gt)) => Expr::Gt,
        Some(Ok(Token::NEq)) => Expr::NEq,
        Some(Ok(Token::LtEq)) => Expr::LtEq,
        Some(Ok(Token::GtEq)) => Expr::GtEq,
        Some(Ok(_)) | None => return parse_add(lhs, parser), // try next level up
        Some(Err(error)) => return Err(vec![error]),
    };

    parser.take_next(); // consume op
    let rhs = parse_atom(parser)?;
    let rhs = parse_add(rhs, parser)?; // parse higher precedence
    let span = parser.span(lhs.data.start()..rhs.data.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(span))
}

pub fn parse_and<T: TokenStream>(
    lhs: ExprNode<StreamSpan<T>>,
    parser: &mut TokenParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<PError<T>>> {
    let op = match parser.peek_next() {
        Some(Ok(Token::And)) => Expr::And,
        Some(Ok(_)) | None => return parse_relation(lhs, parser), // try next level up
        Some(Err(error)) => return Err(vec![error]),
    };

    parser.take_next(); // consume op
    let rhs = parse_atom(parser)?;
    let rhs = parse_relation(rhs, parser)?; // parse higher precedence
    let span = parser.span(lhs.data.start()..rhs.data.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(span))
}

pub fn parse_or<T: TokenStream>(
    lhs: ExprNode<StreamSpan<T>>,
    parser: &mut TokenParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<PError<T>>> {
    let op = match parser.peek_next() {
        Some(Ok(Token::Or)) => Expr::Or,
        Some(Ok(_)) | None => return parse_and(lhs, parser), // try next level up
        Some(Err(error)) => return Err(vec![error]),
    };

    parser.take_next(); // consume op
    let rhs = parse_atom(parser)?;
    let rhs = parse_and(rhs, parser)?; // parse higher precedence
    let span = parser.span(lhs.data.start()..rhs.data.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(span))
}

pub fn parse_ternary<T: TokenStream>(
    cond: ExprNode<StreamSpan<T>>,
    parser: &mut TokenParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<PError<T>>> {
    // parse the question mark
    match parser.peek_next() {
        Some(Ok(Token::Question)) => (),
        _ => return parse_or(cond, parser), // try next level up
    };
    parser.take_next(); // consume question

    // parse the pass condition
    let lhs = parse_atom(parser)?;
    let pass = parse_or(lhs, parser)?; // parse higher precedence

    // parse the colon seperator
    parser
        .take_expect(Some(&Token::Colon))
        .map_err(|e| vec![e])?;

    // parse the fail condition
    let lhs = parse_atom(parser)?;
    let fail = parse_or(lhs, parser)?; // parse higher precedence

    // build the ternary
    let span = parser.span(cond.data.start()..fail.data.end());
    Ok(Expr::Ternary {
        cond: Box::new(cond),
        pass: Box::new(pass),
        fail: Box::new(fail),
    }
    .build_node(span))
}

pub fn parse_walrus<T: TokenStream>(
    lhs: ExprNode<StreamSpan<T>>,
    parser: &mut TokenParser<T>,
) -> Result<ExprNode<StreamSpan<T>>, Vec<PError<T>>> {
    let op = match parser.peek_next() {
        Some(Ok(Token::Walrus)) => Expr::Walrus,
        Some(Ok(_)) | None => return parse_ternary(lhs, parser), // try next level up
        Some(Err(error)) => return Err(vec![error]),
    };

    parser.take_next(); // consume op
    let rhs = parse_atom(parser)?;
    let rhs = parse_ternary(rhs, parser)?; // parse higher precedence
    let span = parser.span(lhs.data.start()..rhs.data.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(span))
}
