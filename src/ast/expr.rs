use crate::{
    lexer::Token,
    parser::{error::PResult, Node, ParserSource},
};

#[derive(Debug)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Neg(Box<Node<Expr>>),
    Not(Box<Node<Expr>>),
}

impl Expr {
    fn parse_int(str: impl AsRef<str>) -> Result<i64, String> {
        match str.as_ref().parse() {
            Ok(value) => Ok(value),
            Err(e) => Err(format!("{e}")),
        }
    }

    fn parse_float(str: impl AsRef<str>) -> Result<f64, String> {
        match str.as_ref().parse() {
            Ok(value) => Ok(value),
            Err(e) => Err(format!("{e}")),
        }
    }

    pub fn parse_atom<'a>(source: &mut impl ParserSource<'a>) -> PResult<Node<Self>> {
        let mut builder = source.node_builder();
        match builder.take() {
            Some((Token::Int(str), _)) => match Self::parse_int(str) {
                Ok(value) => Ok(builder.build(Expr::Int(value))),
                Err(message) => Err(message),
            },
            Some((Token::Float(str), _)) => match Self::parse_float(str) {
                Ok(value) => Ok(builder.build(Expr::Float(value))),
                Err(message) => Err(message),
            },
            Some((Token::Bool(bool), _)) => Ok(builder.build(Expr::Bool(bool))),
            Some((Token::String(str), _)) => Ok(builder.build(Expr::String(str.to_string()))),
            Some((Token::Sub, _)) => match builder.peek() {
                Some((Token::Int(str), _)) => match Self::parse_int(format!("-{str}")) {
                    Ok(value) => Ok(builder.build(Expr::Int(value))),
                    Err(message) => Err(message),
                },
                Some((Token::Float(str), _)) => match Self::parse_float(format!("-{str}")) {
                    Ok(value) => Ok(builder.build(Expr::Float(value))),
                    Err(message) => Err(message),
                },
                Some(_) => {
                    let nested = Self::parse_atom(&mut builder)?;
                    Ok(builder.build(Expr::Neg(Box::new(nested))))
                }
                None => Err(format!("expected expr after '-', found nothing")),
            },
            Some((Token::Not, _)) => {
                let nested = Self::parse_atom(&mut builder)?;
                Ok(builder.build(Expr::Not(Box::new(nested))))
            }
            Some((token, _)) => Err(format!("invalid token {token}")),
            None => Err(format!("expected expr, found nothing")),
        }
    }
}