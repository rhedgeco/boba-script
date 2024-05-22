use crate::parser::{PError, PResult, Span};

use super::{Expr, Node};

pub fn parse_int(span: Span, str: impl AsRef<str>) -> PResult<Node<Expr>> {
    match str.as_ref().parse() {
        Err(error) => Err(PError::ParseIntError { error, span }),
        Ok(value) => Ok(Node::new(span, Expr::Int(value))),
    }
}

pub fn parse_float(span: Span, str: impl AsRef<str>) -> PResult<Node<Expr>> {
    match str.as_ref().parse() {
        Err(error) => Err(PError::ParseFloatError { error, span }),
        Ok(value) => Ok(Node::new(span, Expr::Float(value))),
    }
}
