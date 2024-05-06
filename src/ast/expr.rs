use std::iter::Peekable;

use crate::{
    ast::{Color, ErrorLabel},
    token::Ident,
    Token,
};

use super::{Node, ParserError, TokenIter, TokenParser, Value};

#[derive(Debug)]
pub enum Expr {
    Var(Ident),
    Int(i64),
    Float(f64),
    Neg(Box<Node<Expr>>),
    Add(Box<Node<Expr>>, Box<Node<Expr>>),
    Sub(Box<Node<Expr>>, Box<Node<Expr>>),
    Mul(Box<Node<Expr>>, Box<Node<Expr>>),
    Div(Box<Node<Expr>>, Box<Node<Expr>>),
    Pow(Box<Node<Expr>>, Box<Node<Expr>>),
}

impl TokenParser for Expr {
    type Output = Self;

    fn parse(tokens: &mut Peekable<impl TokenIter>) -> Result<Node<Self::Output>, ParserError> {
        fn parse_atom(tokens: &mut Peekable<impl TokenIter>) -> Result<Node<Expr>, ParserError> {
            match tokens.next() {
                Some((Token::Ident(ident), span)) => Ok(Node::new(span, Expr::Var(ident.clone()))),
                Some((Token::Int(int), span)) => Ok(Node::new(span, Expr::Int(int))),
                Some((Token::Float(float), span)) => Ok(Node::new(span, Expr::Float(float))),
                Some((Token::Add, _)) => Ok(parse_atom(tokens)?),
                Some((Token::Sub, span)) => {
                    Ok(Node::new(span, Expr::Neg(Box::new(parse_atom(tokens)?))))
                }
                Some((Token::OpenParen, _)) => {
                    let tokens = tokens
                        .take_while(|(t, _)| t != &Token::CloseParen)
                        .collect::<Vec<_>>();

                    Expr::parse(&mut tokens.into_iter().peekable())
                }
                Some((token, span)) => Err(ParserError {
                    message: format!("Unexpected token found while parsing expression"),
                    labels: vec![ErrorLabel {
                        message: format!("found token '{token:?}'"),
                        color: Color::Red,
                        span: span.clone(),
                    }],
                }),
                None => Err(ParserError {
                    message: format!("Reached end of input while parsing expression"),
                    labels: vec![],
                }),
            }
        }

        fn parse_pow(
            lhs: Node<Expr>,
            tokens: &mut Peekable<impl TokenIter>,
        ) -> Result<Node<Expr>, ParserError> {
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
        ) -> Result<Node<Expr>, ParserError> {
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
        ) -> Result<Node<Expr>, ParserError> {
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
                    return Err(ParserError {
                        message: format!("Unexpected token found while parsing expression"),
                        labels: vec![ErrorLabel {
                            message: format!("2 found token '{token:?}'"),
                            color: Color::Red,
                            span: span.clone(),
                        }],
                    })
                }
            };
        }
    }
}

impl Expr {
    pub fn eval(&self, vars: &[(Ident, Value)]) -> Option<Value> {
        match self {
            Self::Int(v) => Some(Value::Int(*v)),
            Self::Float(v) => Some(Value::Float(*v)),
            Self::Neg(expr) => Some(-expr.eval(vars)?),
            Self::Var(ident) => vars.iter().rev().find_map(|(id, val)| {
                if id.as_str() == ident.as_str() {
                    Some(*val)
                } else {
                    None
                }
            }),
            Self::Add(lhs, rhs) => Some(lhs.eval(vars)? + rhs.eval(vars)?),
            Self::Sub(lhs, rhs) => Some(lhs.eval(vars)? - rhs.eval(vars)?),
            Self::Mul(lhs, rhs) => Some(lhs.eval(vars)? * rhs.eval(vars)?),
            Self::Div(lhs, rhs) => Some(lhs.eval(vars)? / rhs.eval(vars)?),
            Self::Pow(lhs, rhs) => Some(lhs.eval(vars)?.pow(rhs.eval(vars)?)),
        }
    }
}
