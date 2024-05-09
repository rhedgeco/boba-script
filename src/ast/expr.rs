use std::iter::Peekable;

use crate::{
    error::{Color, Label},
    lexer::{token::Span, Ident, Token},
    LangError,
};

use super::{Node, TokenIter, TokenParser};

#[derive(Debug)]
pub enum Expr {
    Var(Ident),
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Neg(Box<Node<Expr>>),
    Not(Box<Node<Expr>>),
    Add(Box<Node<Expr>>, Box<Node<Expr>>),
    Sub(Box<Node<Expr>>, Box<Node<Expr>>),
    Mul(Box<Node<Expr>>, Box<Node<Expr>>),
    Div(Box<Node<Expr>>, Box<Node<Expr>>),
    Pow(Box<Node<Expr>>, Box<Node<Expr>>),
}

impl TokenParser for Expr {
    type Output = Self;

    fn parse(tokens: &mut Peekable<impl TokenIter>) -> Result<Node<Self::Output>, LangError> {
        // create helper function for parsing floats
        fn parse_float(span: Span, float: impl AsRef<str>) -> Result<Node<Expr>, LangError> {
            match float.as_ref().parse::<f64>() {
                Ok(float) => Ok(Node::new(span, Expr::Float(float))),
                Err(e) => Err(LangError::new("Parse Float Error").label(Label::new(
                    format!("{e}"),
                    Color::Red,
                    span,
                ))),
            }
        }

        // create helper function for parsing integers
        fn parse_int(span: Span, int: impl AsRef<str>) -> Result<Node<Expr>, LangError> {
            match int.as_ref().parse::<i64>() {
                Ok(int) => Ok(Node::new(span, Expr::Int(int))),
                Err(e) => match e.kind() {
                    std::num::IntErrorKind::PosOverflow => Err(format!(
                        "Integer is too big. Must be at max 9,223,372,036,854,775,807"
                    )),
                    std::num::IntErrorKind::NegOverflow => Err(format!(
                        "Integer is too small. Must be at min -9,223,372,036,854,775,808"
                    )),
                    _ => Err(format!("{e}")),
                }
                .map_err(|message| {
                    LangError::new("Parse Integer Error").label(Label::new(
                        message,
                        Color::Red,
                        span,
                    ))
                }),
            }
        }

        // create helper function for parsing pow operators
        fn parse_pow(
            lhs: Node<Expr>,
            tokens: &mut Peekable<impl TokenIter>,
        ) -> Result<Node<Expr>, LangError> {
            let op = match tokens.peek() {
                Some((Token::Pow, _)) => Expr::Pow as fn(_, _) -> _,
                _ => return Ok(lhs),
            };

            tokens.next(); // consume operator
            let rhs = parse_atom(tokens)?;
            let span = lhs.span().start..rhs.span().end;
            let new_lhs = Node::new(span, op(Box::new(lhs), Box::new(rhs)));
            parse_pow(new_lhs, tokens)
        }

        // create helper function for parsing mul/div operators
        fn parse_mul_div(
            lhs: Node<Expr>,
            tokens: &mut Peekable<impl TokenIter>,
        ) -> Result<Node<Expr>, LangError> {
            let op = match tokens.peek() {
                Some((Token::Mul, _)) => Expr::Mul as fn(_, _) -> _,
                Some((Token::Div, _)) => Expr::Div as fn(_, _) -> _,
                _ => return Ok(lhs),
            };

            tokens.next(); // consume operator
            let mut rhs = parse_atom(tokens)?;
            rhs = parse_pow(rhs, tokens)?; // ensure pow comes first in op order
            let span = lhs.span().start..rhs.span().end;
            let new_lhs = Node::new(span, op(Box::new(lhs), Box::new(rhs)));
            parse_mul_div(new_lhs, tokens)
        }

        // create helper function for parsing add/sub operators
        fn parse_add_sub(
            lhs: Node<Expr>,
            tokens: &mut Peekable<impl TokenIter>,
        ) -> Result<Node<Expr>, LangError> {
            let op = match tokens.peek() {
                Some((Token::Add, _)) => Expr::Add as fn(_, _) -> _,
                Some((Token::Sub, _)) => Expr::Sub as fn(_, _) -> _,
                _ => return Ok(lhs),
            };

            tokens.next(); // consume operator
            let mut rhs = parse_atom(tokens)?;
            rhs = parse_mul_div(rhs, tokens)?; // ensure mul/div comes first in op order
            let span = lhs.span().start..rhs.span().end;
            let new_lhs = Node::new(span, op(Box::new(lhs), Box::new(rhs)));
            parse_add_sub(new_lhs, tokens)
        }

        // create helper function for parsing all atomic expressions
        fn parse_atom(tokens: &mut Peekable<impl TokenIter>) -> Result<Node<Expr>, LangError> {
            match tokens.next() {
                // parse variables
                Some((Token::Ident(ident), span)) => Ok(Node::new(span, Expr::Var(ident.clone()))),
                // parse bools
                Some((Token::Bool(bool), span)) => Ok(Node::new(span, Expr::Bool(bool))),
                // parse ints
                Some((Token::Int(int), span)) => parse_int(span, int),
                // parse floats
                Some((Token::Float(float), span)) => parse_float(span, float),
                // parse strings
                Some((Token::String(string), span)) => Ok(Node::new(span, Expr::String(string))),
                // parse prefixed positives
                Some((Token::Add, _)) => Ok(parse_atom(tokens)?),
                // parse prefixed negatives
                Some((Token::Sub, span)) => match tokens.peek() {
                    // parse negative integers
                    Some((Token::Int(int), ispan)) => {
                        let parsed = parse_int(span.start..ispan.end, format!("-{int}"));
                        tokens.next(); //consume parsed token
                        parsed
                    }
                    // parse negative floats
                    Some((Token::Float(float), fspan)) => {
                        let parsed = parse_float(span.start..fspan.end, format!("-{float}"));
                        tokens.next(); //consume parsed token
                        parsed
                    }
                    // parse the negation of any other atomic expression
                    _ => {
                        let atom = parse_atom(tokens)?;
                        let span = span.start..atom.span().end;
                        Ok(Node::new(span, Expr::Neg(Box::new(atom))))
                    }
                },
                // parse prefixed not operators
                Some((Token::Not, span)) => {
                    let atom = parse_atom(tokens)?;
                    let span = span.start..atom.span().end;
                    Ok(Node::new(span, Expr::Not(Box::new(atom))))
                }
                // parse parentheses delimited expressions
                Some((Token::OpenParen, span)) => {
                    let mut open_count = 1; // create counter for open parens
                    let tokens = tokens // take tokens and count parens untill all are closed
                        .take_while(|(t, _)| {
                            match t {
                                Token::OpenParen => open_count += 1,
                                Token::CloseParen => open_count -= 1,
                                _ => (),
                            }
                            open_count > 0
                        })
                        .collect::<Vec<_>>();

                    // check if we collected any tokens
                    match tokens.last() {
                        Some((_, last_span)) if open_count > 0 => {
                            Err(LangError::new("Unclosed brace found while parsing")
                                .label(Label::new("open brace found here", Color::Cyan, span))
                                .label(Label::new(
                                    "expression ended here",
                                    Color::Red,
                                    last_span.clone(),
                                )))
                        }
                        None if open_count > 0 => Err(LangError::new(
                            "Unexpected end of expression",
                        )
                        .label(Label::new(
                            "expected expression, found '('",
                            Color::Red,
                            span,
                        ))),
                        None => Err(LangError::new("Empty braces found while parsing").label(
                            Label::new(
                                "expected expression found '()'",
                                Color::Red,
                                span.start..span.end + 1,
                            ),
                        )),
                        _ => Expr::parse(&mut tokens.into_iter().peekable()),
                    }
                }
                // fail if no other atomics were found
                Some((token, span)) => Err(LangError::new(
                    "Unexpected token found while parsing expression",
                )
                .label(Label::new(
                    format!("unexpected token '{token}'"),
                    Color::Red,
                    span.clone(),
                ))),
                // fail if no tokens were found
                None => Err(LangError::new(
                    "Reached end of input while parsing expression",
                )),
            }
        }

        // parse the initial expression atomic
        let mut expr = parse_atom(tokens)?;

        // parse all following operators in the expression
        while let Some((token, span)) = tokens.peek() {
            // fold everything into the original expression variable
            expr = match token {
                // parse pow
                Token::Pow => parse_pow(expr, tokens)?,
                // parse mul/div
                Token::Mul | Token::Div => parse_mul_div(expr, tokens)?,
                // parse add/sub
                Token::Add | Token::Sub => parse_add_sub(expr, tokens)?,
                // fail if no operator found
                token => {
                    return Err(
                        LangError::new("Unexpected token found while parsing expression").label(
                            Label::new(
                                format!("unexpected token '{token}'"),
                                Color::Red,
                                span.clone(),
                            ),
                        ),
                    )
                }
            };
        }

        // return the final expression
        Ok(expr)
    }
}
