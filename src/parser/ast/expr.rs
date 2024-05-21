use crate::parser::{PError, PResult, Span, Token, TokenLine};

use super::{
    span::{SpanPrefix, Spanned, Spanner},
    utils,
};

pub enum Expr {
    // values
    Var(Spanner<String>),
    Bool(Spanner<bool>),
    Int(Spanner<i64>),
    Float(Spanner<f64>),
    String(Spanner<String>),

    // math operations
    Neg(SpanPrefix<Box<Expr>>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),

    // boolean operations
    Not(SpanPrefix<Box<Expr>>),
    Eq(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    NEq(Box<Expr>, Box<Expr>),
    LtEq(Box<Expr>, Box<Expr>),
    GtEq(Box<Expr>, Box<Expr>),

    // walrus
    Walrus(Spanner<String>, Box<Expr>),

    // ternary
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
}

impl Spanned for Expr {
    fn span(&self) -> Span {
        match self {
            Expr::Var(v) => v.span(),
            Expr::Bool(v) => v.span(),
            Expr::Int(v) => v.span(),
            Expr::Float(v) => v.span(),
            Expr::String(v) => v.span(),
            Expr::Neg(e) => e.span(),
            Expr::Add(e1, e2) => e1.span().start..e2.span().end,
            Expr::Sub(e1, e2) => e1.span().start..e2.span().end,
            Expr::Mul(e1, e2) => e1.span().start..e2.span().end,
            Expr::Div(e1, e2) => e1.span().start..e2.span().end,
            Expr::Mod(e1, e2) => e1.span().start..e2.span().end,
            Expr::Pow(e1, e2) => e1.span().start..e2.span().end,
            Expr::Not(e) => e.span(),
            Expr::Eq(e1, e2) => e1.span().start..e2.span().end,
            Expr::Lt(e1, e2) => e1.span().start..e2.span().end,
            Expr::Gt(e1, e2) => e1.span().start..e2.span().end,
            Expr::NEq(e1, e2) => e1.span().start..e2.span().end,
            Expr::LtEq(e1, e2) => e1.span().start..e2.span().end,
            Expr::GtEq(e1, e2) => e1.span().start..e2.span().end,
            Expr::Walrus(v, e) => v.span().start..e.span().end,
            Expr::Ternary(e1, _, e2) => e1.span().start..e2.span().end,
        }
    }
}

impl Expr {
    pub fn parse_atom(tokens: &mut TokenLine) -> PResult<Self> {
        match tokens.next_expect("expression")? {
            // values
            (Token::Ident(str), span) => Ok(Expr::Var(Spanner::new(span, str.into()))),
            (Token::Bool(bool), span) => Ok(Expr::Bool(Spanner::new(span, bool))),
            (Token::UInt(str), span) => Ok(Expr::Int(utils::parse_int(span, str)?)),
            (Token::UFloat(str), span) => Ok(Expr::Float(utils::parse_float(span, str)?)),
            (Token::String(str), span) => Ok(Expr::String(Spanner::new(span, str.into()))),

            // prefix expressions
            (Token::Not, span) => {
                let nested = Self::parse_atom(tokens)?;
                let offset = nested.span().start - span.start;
                Ok(Expr::Not(SpanPrefix::new(offset, Box::new(nested))))
            }
            (Token::Sub, span) => match tokens.peek_expect("expression")? {
                (Token::UInt(str), sub_span) => Ok(Expr::Int(utils::parse_int(
                    span.start..sub_span.end,
                    format!("-{str}"),
                )?)),
                (Token::UFloat(str), sub_span) => Ok(Expr::Float(utils::parse_float(
                    span.start..sub_span.end,
                    format!("-{str}"),
                )?)),
                _ => {
                    let nested = Self::parse_atom(tokens)?;
                    let offset = nested.span().start - span.start;
                    Ok(Expr::Neg(SpanPrefix::new(offset, Box::new(nested))))
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
