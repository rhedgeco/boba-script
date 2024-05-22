use crate::parser::{PError, PResult, Token, TokenLine};

use super::{utils, Node};

pub enum Expr {
    // values
    Var(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),

    // math operations
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),

    // boolean operations
    Not(Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    NEq(Box<Expr>, Box<Expr>),
    LtEq(Box<Expr>, Box<Expr>),
    GtEq(Box<Expr>, Box<Expr>),

    // walrus
    Walrus(Node<String>, Box<Expr>),

    // ternary
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
}

impl Expr {
    pub fn parse_atom(tokens: &mut TokenLine) -> PResult<Node<Self>> {
        match tokens.next_expect("expression")? {
            // values
            (Token::Ident(str), span) => Ok(Node::new(span, Expr::Var(str.into()))),
            (Token::Bool(bool), span) => Ok(Node::new(span, Expr::Bool(bool))),
            (Token::UInt(str), span) => Ok(utils::parse_int(span, str)?),
            (Token::UFloat(str), span) => Ok(utils::parse_float(span, str)?),
            (Token::String(str), span) => Ok(Node::new(span, Expr::String(str.into()))),

            // prefix expressions
            (Token::Not, span) => {
                let nested = Self::parse_atom(tokens)?;
                let span = span.start..nested.span().end;
                Ok(Node::new(span, Expr::Not(Box::new(nested.into_inner()))))
            }
            (Token::Sub, span) => match tokens.peek_expect("expression")? {
                (Token::UInt(str), num_span) => Ok(utils::parse_int(
                    span.start..num_span.end,
                    format!("-{str}"),
                )?),
                (Token::UFloat(str), num_span) => Ok(utils::parse_float(
                    span.start..num_span.end,
                    format!("-{str}"),
                )?),
                _ => {
                    let nested = Self::parse_atom(tokens)?;
                    let span = span.start..nested.span().end;
                    Ok(Node::new(span, Expr::Neg(Box::new(nested.into_inner()))))
                }
            },

            // braced expressions
            // (Token::OpenParen, span) => {
            // }

            // error case
            (token, span) => Err(PError::UnexpectedToken {
                expected: format!("expression"),
                found: format!("'{token}'"),
                span,
            }),
        }
    }
}
