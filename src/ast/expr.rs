use std::iter::Peekable;

use crate::{
    error::{Color, Label},
    token::Ident,
    LangError, Token,
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
        fn parse_atom(tokens: &mut Peekable<impl TokenIter>) -> Result<Node<Expr>, LangError> {
            match tokens.next() {
                Some((Token::Ident(ident), span)) => Ok(Node::new(span, Expr::Var(ident.clone()))),
                Some((Token::Bool(bool), span)) => Ok(Node::new(span, Expr::Bool(bool))),
                Some((Token::Int(int), span)) => Ok(Node::new(span, Expr::Int(int))),
                Some((Token::Float(float), span)) => Ok(Node::new(span, Expr::Float(float))),
                Some((Token::String(string), span)) => Ok(Node::new(span, Expr::String(string))),
                Some((Token::Add, _)) => Ok(parse_atom(tokens)?),
                Some((Token::Sub, span)) => {
                    let atom = parse_atom(tokens)?;
                    let span = span.start..atom.span().end;
                    Ok(Node::new(span, Expr::Neg(Box::new(atom))))
                }
                Some((Token::Not, span)) => {
                    let atom = parse_atom(tokens)?;
                    let span = span.start..atom.span().end;
                    Ok(Node::new(span, Expr::Not(Box::new(atom))))
                }
                Some((Token::OpenParen, span)) => {
                    let mut open_count = 1;
                    let tokens = tokens
                        .take_while(|(t, _)| {
                            match t {
                                Token::OpenParen => open_count += 1,
                                Token::CloseParen => open_count -= 1,
                                _ => (),
                            }
                            open_count > 0
                        })
                        .collect::<Vec<_>>();

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
                Some((token, span)) => Err(LangError::new(
                    "Unexpected token found while parsing expression",
                )
                .label(Label::new(
                    format!("found token '{token:?}'"),
                    Color::Red,
                    span.clone(),
                ))),
                None => Err(LangError::new(
                    "Reached end of input while parsing expression",
                )),
            }
        }

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

        let mut expr = parse_atom(tokens)?;
        loop {
            expr = match tokens.peek() {
                None => return Ok(expr),
                Some((Token::Pow, _)) => parse_pow(expr, tokens)?,
                Some((Token::Mul, _)) | Some((Token::Div, _)) => parse_mul_div(expr, tokens)?,
                Some((Token::Add, _)) | Some((Token::Sub, _)) => parse_add_sub(expr, tokens)?,
                Some((token, span)) => {
                    return Err(
                        LangError::new("Unexpected token found while parsing expression").label(
                            Label::new(
                                format!("found token '{token:?}'"),
                                Color::Red,
                                span.clone(),
                            ),
                        ),
                    )
                }
            };
        }
    }
}
