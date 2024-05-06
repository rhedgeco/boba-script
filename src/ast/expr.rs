use std::iter::Peekable;

use crate::{
    ast::{Color, ErrorLabel},
    token::Ident,
    Token,
};

use super::{ParserError, TokenIter, TokenParser, Value};

#[derive(Debug)]
pub enum Expr {
    Var(Ident),
    Int(i64),
    Float(f64),
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
}

impl TokenParser for Expr {
    type Output = Self;

    fn parse(tokens: &mut Peekable<impl TokenIter>) -> Result<Self::Output, ParserError> {
        fn parse_atom(tokens: &mut Peekable<impl TokenIter>) -> Result<Expr, ParserError> {
            match tokens.next() {
                Some((Token::Ident(ident), _)) => Ok(Expr::Var(ident.clone())),
                Some((Token::Int(int), _)) => Ok(Expr::Int(int)),
                Some((Token::Float(float), _)) => Ok(Expr::Float(float)),
                Some((Token::Add, _)) => Ok(parse_atom(tokens)?),
                Some((Token::Sub, _)) => Ok(Expr::Neg(Box::new(parse_atom(tokens)?))),
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

        fn parse_mul_div(
            lhs: Expr,
            tokens: &mut Peekable<impl TokenIter>,
        ) -> Result<Expr, ParserError> {
            let op = match tokens.peek() {
                Some((Token::Mul, _)) => Expr::Mul as fn(_, _) -> _,
                Some((Token::Div, _)) => Expr::Div as fn(_, _) -> _,
                _ => return Ok(lhs),
            };

            tokens.next(); // consume operator
            let rhs = parse_atom(tokens)?;
            let new_lhs = op(Box::new(lhs), Box::new(rhs));
            parse_mul_div(new_lhs, tokens)
        }

        fn parse_add_sub(
            lhs: Expr,
            tokens: &mut Peekable<impl TokenIter>,
        ) -> Result<Expr, ParserError> {
            let op = match tokens.peek() {
                Some((Token::Add, _)) => Expr::Add as fn(_, _) -> _,
                Some((Token::Sub, _)) => Expr::Sub as fn(_, _) -> _,
                _ => return Ok(lhs),
            };

            tokens.next(); // consume operator
            let mut rhs = parse_atom(tokens)?;
            rhs = parse_mul_div(rhs, tokens)?; // ensure mul/div comes first in op order
            let new_lhs = op(Box::new(lhs), Box::new(rhs));
            parse_add_sub(new_lhs, tokens)
        }

        let mut expr = parse_atom(tokens)?;
        loop {
            expr = match tokens.peek() {
                None => return Ok(expr),
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
        }
    }
}
