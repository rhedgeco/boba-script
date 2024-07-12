use boba_script_core::ast::{node::Builder, Expr, ExprNode};

use crate::{
    error::PError, stream::SourceSpan, ConsumeEnd, ConsumeFlag, ParseError, Token, TokenLine,
    TokenStream,
};

pub fn parse<T: TokenStream>(
    line: &mut TokenLine<T>,
) -> Result<ExprNode<T::Source>, Vec<PError<T>>> {
    let lhs = parse_atom(line)?;
    parse_with_lhs(lhs, line)
}

pub fn parse_atom<T: TokenStream>(
    line: &mut TokenLine<T>,
) -> Result<ExprNode<T::Source>, Vec<PError<T>>> {
    line.take_guard(|token, line| match token {
        // VALUES
        Some(Token::None) => Ok(Expr::None.build_node(line.token_source())),
        Some(Token::Bool(value)) => Ok(Expr::Bool(value).build_node(line.token_source())),
        Some(Token::Int(value)) => Ok(Expr::Int(value).build_node(line.token_source())),
        Some(Token::Float(value)) => Ok(Expr::Float(value).build_node(line.token_source())),
        Some(Token::String(value)) => Ok(Expr::String(value).build_node(line.token_source())),

        // VARS / CALLS
        Some(Token::Ident(ident)) => Ok(Expr::Var(ident).build_node(line.token_source())),

        // PARENTHESIS AND TUPLES
        Some(Token::OpenParen) => {
            // save the open paren span
            let start = line.token_start();

            // parse all tuple parts if any
            let mut exprs = Vec::new();
            let expr = loop {
                // try parsing an inner expression
                let result = line.guard_else(
                    |line| {
                        // parse expression
                        let inner = parse(line)?;

                        // then check for a comma or closing paren
                        let end = line.take_guard(|token, line| match token {
                            // a paren will tell the loop it is complete
                            Some(Token::CloseParen) => Ok(true),
                            // and a comma will tell the loop to continue
                            Some(Token::Comma) => Ok(false),
                            // otherwise it is an invalid token
                            token => Err(vec![ParseError::UnexpectedInput {
                                expect: "',' or ')'".into(),
                                found: token,
                                source: line.token_source(),
                            }]),
                        })?;

                        // then return the inner expression
                        Ok((inner, end))
                    },
                    |errors| {
                        // consume until the end of the inner expression

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
                    let source = line.build_source(start..line.token_end());
                    Ok(Expr::Tuple(exprs).build_node(source))
                }
            }
        }

        // FAILURE CASE
        token => Err(vec![ParseError::UnexpectedInput {
            expect: "expression".into(),
            found: token,
            source: line.token_source(),
        }]),
    })
}

pub fn parse_with_lhs<T: TokenStream>(
    mut lhs: ExprNode<T::Source>,
    line: &mut TokenLine<T>,
) -> Result<ExprNode<T::Source>, Vec<PError<T>>> {
    // keep parsing operators until an invalid operator is found
    loop {
        lhs = match line.peek_token() {
            Some(Ok(token)) => match token {
                Token::Pow => parse_pow(lhs, line)?,
                Token::Mul | Token::Div | Token::Modulo => parse_mul(lhs, line)?,
                Token::Add | Token::Sub => parse_add(lhs, line)?,
                Token::Eq | Token::Lt | Token::Gt | Token::NEq | Token::LtEq | Token::GtEq => {
                    parse_relation(lhs, line)?
                }
                Token::And => parse_and(lhs, line)?,
                Token::Or => parse_or(lhs, line)?,
                Token::Question => parse_ternary(lhs, line)?,
                Token::Walrus => parse_walrus(lhs, line)?,
                _ => return Ok(lhs),
            },
            _ => return Ok(lhs),
        }
    }
}

pub fn parse_pow<T: TokenStream>(
    lhs: ExprNode<T::Source>,
    line: &mut TokenLine<T>,
) -> Result<ExprNode<T::Source>, Vec<PError<T>>> {
    let op = match line.peek_token() {
        Some(Ok(Token::Pow)) => Expr::Pow,
        _ => return Ok(lhs),
    };

    line.consume_token(); // consume op
    let rhs = parse_atom(line)?;
    let rhs = parse_pow(rhs, line)?; // parse right to left
    let source = line.build_source(lhs.source.start()..rhs.source.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(source))
}

pub fn parse_mul<T: TokenStream>(
    lhs: ExprNode<T::Source>,
    line: &mut TokenLine<T>,
) -> Result<ExprNode<T::Source>, Vec<PError<T>>> {
    let op = match line.peek_token() {
        Some(Ok(Token::Mul)) => Expr::Mul,
        Some(Ok(Token::Div)) => Expr::Div,
        Some(Ok(Token::Modulo)) => Expr::Modulo,
        Some(Err(_)) => return Ok(lhs),
        // try the next precedence level
        _ => return parse_pow(lhs, line),
    };

    line.consume_token(); // consume op
    let rhs = parse_atom(line)?;
    let rhs = parse_pow(rhs, line)?; // parse higher precedence on rhs
    let source = line.build_source(lhs.source.start()..rhs.source.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(source))
}

pub fn parse_add<T: TokenStream>(
    lhs: ExprNode<T::Source>,
    line: &mut TokenLine<T>,
) -> Result<ExprNode<T::Source>, Vec<PError<T>>> {
    let op = match line.peek_token() {
        Some(Ok(Token::Add)) => Expr::Add,
        Some(Ok(Token::Sub)) => Expr::Sub,
        Some(Err(_)) => return Ok(lhs),
        // try the next precedence level
        _ => return parse_mul(lhs, line),
    };

    line.consume_token(); // consume op
    let rhs = parse_atom(line)?;
    let rhs = parse_mul(rhs, line)?; // parse higher precedence on rhs
    let source = line.build_source(lhs.source.start()..rhs.source.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(source))
}

pub fn parse_relation<T: TokenStream>(
    lhs: ExprNode<T::Source>,
    line: &mut TokenLine<T>,
) -> Result<ExprNode<T::Source>, Vec<PError<T>>> {
    let op = match line.peek_token() {
        Some(Ok(Token::Eq)) => Expr::Eq,
        Some(Ok(Token::Lt)) => Expr::Lt,
        Some(Ok(Token::Gt)) => Expr::Gt,
        Some(Ok(Token::NEq)) => Expr::NEq,
        Some(Ok(Token::LtEq)) => Expr::LtEq,
        Some(Ok(Token::GtEq)) => Expr::GtEq,
        Some(Err(_)) => return Ok(lhs),
        // try the next precedence level
        _ => return parse_add(lhs, line),
    };

    line.consume_token(); // consume op
    let rhs = parse_atom(line)?;
    let rhs = parse_add(rhs, line)?; // parse higher precedence on rhs
    let source = line.build_source(lhs.source.start()..rhs.source.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(source))
}

pub fn parse_and<T: TokenStream>(
    lhs: ExprNode<T::Source>,
    line: &mut TokenLine<T>,
) -> Result<ExprNode<T::Source>, Vec<PError<T>>> {
    let op = match line.peek_token() {
        Some(Ok(Token::And)) => Expr::And,
        // try the next precedence level
        _ => return parse_relation(lhs, line),
    };

    line.consume_token(); // consume op
    let rhs = parse_atom(line)?;
    let rhs = parse_relation(rhs, line)?; // parse higher precedence on rhs
    let source = line.build_source(lhs.source.start()..rhs.source.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(source))
}

pub fn parse_or<T: TokenStream>(
    lhs: ExprNode<T::Source>,
    line: &mut TokenLine<T>,
) -> Result<ExprNode<T::Source>, Vec<PError<T>>> {
    let op = match line.peek_token() {
        Some(Ok(Token::Or)) => Expr::Or,
        // try the next precedence level
        _ => return parse_and(lhs, line),
    };

    line.consume_token(); // consume op
    let rhs = parse_atom(line)?;
    let rhs = parse_and(rhs, line)?; // parse higher precedence on rhs
    let source = line.build_source(lhs.source.start()..rhs.source.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(source))
}

pub fn parse_ternary<T: TokenStream>(
    cond: ExprNode<T::Source>,
    line: &mut TokenLine<T>,
) -> Result<ExprNode<T::Source>, Vec<PError<T>>> {
    // parse the question mark
    match line.peek_token() {
        Some(Ok(Token::Question)) => (),
        // try the next precedence level
        _ => return parse_or(cond, line),
    };

    // consume the question mark
    line.consume_token();

    // parse the pass expression
    let pass = parse(line)?;

    // parse the colon delimiter
    line.take_exact(Some(&Token::Colon)).map_err(|e| vec![e])?;

    // parse the fail expression
    let fail = parse(line)?;

    // build source and return the ternary
    let source = line.build_source(cond.source.start()..fail.source.end());
    Ok(Expr::Ternary {
        cond: Box::new(cond),
        pass: Box::new(pass),
        fail: Box::new(fail),
    }
    .build_node(source))
}

pub fn parse_walrus<T: TokenStream>(
    lhs: ExprNode<T::Source>,
    line: &mut TokenLine<T>,
) -> Result<ExprNode<T::Source>, Vec<PError<T>>> {
    let op = match line.peek_token() {
        Some(Ok(Token::Walrus)) => Expr::Walrus,
        // try the next precedence level
        _ => return parse_ternary(lhs, line),
    };

    line.consume_token(); // consume op
    let rhs = parse_atom(line)?;
    let rhs = parse_ternary(rhs, line)?; // parse higher precedence on rhs
    let source = line.build_source(lhs.source.start()..rhs.source.end());
    Ok(op(Box::new(lhs), Box::new(rhs)).build_node(source))
}
