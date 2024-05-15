use crate::{
    lexer::Token,
    parser::{report::PResult, Node, ParserSource},
};

#[derive(Debug)]
pub enum Expr {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Neg(Box<Node<Expr>>),
    Not(Box<Node<Expr>>),
    Pow(Box<Node<Expr>>, Box<Node<Expr>>),
    Mul(Box<Node<Expr>>, Box<Node<Expr>>),
    Div(Box<Node<Expr>>, Box<Node<Expr>>),
    Add(Box<Node<Expr>>, Box<Node<Expr>>),
    Sub(Box<Node<Expr>>, Box<Node<Expr>>),
}

impl Expr {
    pub fn parse_atom<'a>(source: &mut impl ParserSource<'a>) -> PResult<Node<Self>> {
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

        let mut builder = source.node_builder();
        match builder.take() {
            Some((Token::Int(str), _)) => match parse_int(str) {
                Ok(value) => Ok(builder.build(Expr::Int(value))),
                Err(message) => Err(message),
            },
            Some((Token::Float(str), _)) => match parse_float(str) {
                Ok(value) => Ok(builder.build(Expr::Float(value))),
                Err(message) => Err(message),
            },
            Some((Token::Bool(bool), _)) => Ok(builder.build(Expr::Bool(bool))),
            Some((Token::String(str), _)) => Ok(builder.build(Expr::String(str.to_string()))),
            Some((Token::Sub, _)) => match builder.peek() {
                Some((Token::Int(str), _)) => match parse_int(format!("-{str}")) {
                    Ok(value) => Ok(builder.build(Expr::Int(value))),
                    Err(message) => Err(message),
                },
                Some((Token::Float(str), _)) => match parse_float(format!("-{str}")) {
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
            Some((Token::OpenParen, _)) => {
                // parse inner expression
                let inner_expr = Self::parse(&mut builder)?;

                // ensure closing paren
                match builder.take() {
                    Some((Token::CloseParen, _)) => Ok(inner_expr),
                    Some((token, _)) => Err(format!("expected ')', found '{token}'")),
                    None => Err(format!("expected ')', found nothing")),
                }
            }
            Some((token, _)) => Err(format!("invalid token {token}")),
            None => Err(format!("expected expr, found nothing")),
        }
    }

    pub fn parse<'a>(source: &mut impl ParserSource<'a>) -> PResult<Node<Self>> {
        fn try_parse_pow<'a>(
            lhs: Node<Expr>,
            source: &mut impl ParserSource<'a>,
        ) -> PResult<Node<Expr>> {
            let op = match source.peek() {
                Some((Token::Pow, _)) => Expr::Pow,
                _ => return Ok(lhs),
            };

            source.take(); // consume token
            let rhs = Expr::parse_atom(source)?;
            let span = lhs.span().start..rhs.span().end;
            Ok(Node::build(span, op(Box::new(lhs), Box::new(rhs))))
        }

        fn try_parse_mul<'a>(
            lhs: Node<Expr>,
            source: &mut impl ParserSource<'a>,
        ) -> PResult<Node<Expr>> {
            let op = match source.peek() {
                Some((Token::Mul, _)) => Expr::Mul,
                Some((Token::Div, _)) => Expr::Div,
                _ => return Ok(lhs),
            };

            source.take(); // consume token
            let rhs = Expr::parse_atom(source)?;
            let rhs = try_parse_pow(rhs, source)?; // ensure op precedence
            let span = lhs.span().start..rhs.span().end;
            Ok(Node::build(span, op(Box::new(lhs), Box::new(rhs))))
        }

        fn try_parse_add<'a>(
            lhs: Node<Expr>,
            source: &mut impl ParserSource<'a>,
        ) -> PResult<Node<Expr>> {
            let op = match source.peek() {
                Some((Token::Add, _)) => Expr::Add,
                Some((Token::Sub, _)) => Expr::Sub,
                _ => return Ok(lhs),
            };

            source.take(); // consume token
            let rhs = Expr::parse_atom(source)?;
            let rhs = try_parse_mul(rhs, source)?; // ensure op precedence
            let span = lhs.span().start..rhs.span().end;
            Ok(Node::build(span, op(Box::new(lhs), Box::new(rhs))))
        }

        // parse initial atom
        let mut lhs = Self::parse_atom(source)?;

        // loop until all ops are handled
        loop {
            lhs = match source.peek() {
                Some((Token::Pow, _)) => try_parse_pow(lhs, source)?,
                Some((Token::Mul, _)) | Some((Token::Div, _)) => try_parse_mul(lhs, source)?,
                Some((Token::Add, _)) | Some((Token::Sub, _)) => try_parse_add(lhs, source)?,
                _ => return Ok(lhs),
            };
        }
    }
}
