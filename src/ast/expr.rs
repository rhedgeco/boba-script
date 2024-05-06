use std::{collections::HashMap, iter::Peekable, ops::Deref};

use crate::{
    ast::{Color, ErrorLabel},
    token::Ident,
    Token,
};

use super::{BobaError, Node, TokenIter, TokenParser, Value};

#[derive(Debug)]
pub enum Expr {
    Var(Ident),
    Int(i64),
    Float(f64),
    String(String),
    Neg(Box<Node<Expr>>),
    Add(Box<Node<Expr>>, Box<Node<Expr>>),
    Sub(Box<Node<Expr>>, Box<Node<Expr>>),
    Mul(Box<Node<Expr>>, Box<Node<Expr>>),
    Div(Box<Node<Expr>>, Box<Node<Expr>>),
    Pow(Box<Node<Expr>>, Box<Node<Expr>>),
}

impl TokenParser for Expr {
    type Output = Self;

    fn parse(tokens: &mut Peekable<impl TokenIter>) -> Result<Node<Self::Output>, BobaError> {
        fn parse_atom(tokens: &mut Peekable<impl TokenIter>) -> Result<Node<Expr>, BobaError> {
            match tokens.next() {
                Some((Token::Ident(ident), span)) => Ok(Node::new(span, Expr::Var(ident.clone()))),
                Some((Token::Int(int), span)) => Ok(Node::new(span, Expr::Int(int))),
                Some((Token::Float(float), span)) => Ok(Node::new(span, Expr::Float(float))),
                Some((Token::String(string), span)) => Ok(Node::new(span, Expr::String(string))),
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
                Some((token, span)) => Err(BobaError {
                    message: format!("Unexpected token found while parsing expression"),
                    labels: vec![ErrorLabel {
                        message: format!("found token '{token:?}'"),
                        color: Color::Red,
                        span: span.clone(),
                    }],
                }),
                None => Err(BobaError {
                    message: format!("Reached end of input while parsing expression"),
                    labels: vec![],
                }),
            }
        }

        fn parse_pow(
            lhs: Node<Expr>,
            tokens: &mut Peekable<impl TokenIter>,
        ) -> Result<Node<Expr>, BobaError> {
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
        ) -> Result<Node<Expr>, BobaError> {
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
        ) -> Result<Node<Expr>, BobaError> {
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
                    return Err(BobaError {
                        message: format!("Unexpected token found while parsing expression"),
                        labels: vec![ErrorLabel {
                            message: format!("found token '{token:?}'"),
                            color: Color::Red,
                            span: span.clone(),
                        }],
                    })
                }
            };
        }
    }
}

impl Node<Expr> {
    pub fn eval(&self, vars: &HashMap<Ident, Value>) -> Result<Value, BobaError> {
        match self.deref() {
            Expr::Int(v) => Ok(Value::Int(*v)),
            Expr::Float(v) => Ok(Value::Float(*v)),
            Expr::String(v) => Ok(Value::String(v.clone())),
            Expr::Neg(expr) => (-expr.eval(vars)?).map_err(|e| e.into_boba(self.span().clone())),
            Expr::Var(ident) => match vars.get(ident) {
                Some(value) => Ok(value.clone()),
                None => Err(BobaError {
                    message: format!("Evaluation error"),
                    labels: vec![ErrorLabel {
                        message: format!("unknown variable '{}'", ident.as_str()),
                        color: Color::Red,
                        span: self.span().clone(),
                    }],
                }),
            },
            Expr::Add(lhs, rhs) => {
                (lhs.eval(vars)? + rhs.eval(vars)?).map_err(|e| e.into_boba(self.span().clone()))
            }
            Expr::Sub(lhs, rhs) => {
                (lhs.eval(vars)? - rhs.eval(vars)?).map_err(|e| e.into_boba(self.span().clone()))
            }
            Expr::Mul(lhs, rhs) => {
                (lhs.eval(vars)? * rhs.eval(vars)?).map_err(|e| e.into_boba(self.span().clone()))
            }
            Expr::Div(lhs, rhs) => {
                (lhs.eval(vars)? / rhs.eval(vars)?).map_err(|e| e.into_boba(self.span().clone()))
            }
            Expr::Pow(lhs, rhs) => lhs
                .eval(vars)?
                .pow(rhs.eval(vars)?)
                .map_err(|e| e.into_boba(self.span().clone())),
        }
    }
}
