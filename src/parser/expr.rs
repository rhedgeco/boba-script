use crate::{
    error::{Color, Label},
    lexer::{token::Span, Ident, Token},
    parser::parser::NodeBuilder,
    BobaError,
};

use super::Node;

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

impl Expr {
    const UNEXPECTED_ERROR: &'static str = "Unexpected token found while parsing expression";

    pub fn parser(builder: &mut NodeBuilder) -> Result<Self, BobaError> {
        Self::parse_closed(builder, None)
    }

    fn parse_closed(
        builder: &mut NodeBuilder,
        close_with: Option<&Token>,
    ) -> Result<Self, BobaError> {
        // create helper function for parsing pow operators
        fn try_parse_pow(
            lhs: Node<Expr>,
            builder: &mut NodeBuilder,
        ) -> Result<Node<Expr>, BobaError> {
            let op = match builder.peek() {
                Some((Token::Pow, _)) => Expr::Pow,
                _ => return Ok(lhs),
            };

            builder.next(); // consume operator
            let rhs = builder.parse(Expr::atom_parser)?;
            let op_span = lhs.span().start..rhs.span().end;
            Ok(Node::new(op_span, op(Box::new(lhs), Box::new(rhs))))
        }

        // create helper function for parsing mul/div operators
        fn try_parse_mul(
            lhs: Node<Expr>,
            builder: &mut NodeBuilder,
        ) -> Result<Node<Expr>, BobaError> {
            let op = match builder.peek() {
                Some((Token::Mul, _)) => Expr::Mul,
                Some((Token::Div, _)) => Expr::Div,
                _ => return Ok(lhs),
            };

            builder.next(); // consume operator
            let rhs = builder.parse(Expr::atom_parser)?;
            let rhs = try_parse_pow(rhs, builder)?; // prefer pow op ordering
            let op_span = lhs.span().start..rhs.span().end;
            Ok(Node::new(op_span, op(Box::new(lhs), Box::new(rhs))))
        }

        fn try_parse_add(
            lhs: Node<Expr>,
            builder: &mut NodeBuilder,
        ) -> Result<Node<Expr>, BobaError> {
            let op = match builder.peek() {
                Some((Token::Add, _)) => Expr::Add,
                Some((Token::Sub, _)) => Expr::Sub,
                _ => return Ok(lhs),
            };

            builder.next(); // consume operator
            let rhs = builder.parse(Expr::atom_parser)?;
            let rhs = try_parse_mul(rhs, builder)?; // prefer pow op ordering
            let op_span = lhs.span().start..rhs.span().end;
            Ok(Node::new(op_span, op(Box::new(lhs), Box::new(rhs))))
        }

        let mut lhs = builder.parse(Expr::atom_parser)?;
        while let Some((token, span)) = builder.peek() {
            lhs = match token {
                Token::Pow => try_parse_pow(lhs, builder)?,
                Token::Mul | Token::Div => try_parse_mul(lhs, builder)?,
                Token::Add | Token::Sub => try_parse_add(lhs, builder)?,
                token if Some(token) == close_with => break,
                token => {
                    return Err(BobaError::new(Self::UNEXPECTED_ERROR).label(Label::new(
                        format!("Expected '+', '-', '*', '/', or '**', but found '{token}'"),
                        Color::Red,
                        span.clone(),
                    )))
                }
            };
        }

        Ok(lhs.into_inner())
    }

    pub fn atom_parser(builder: &mut NodeBuilder) -> Result<Expr, BobaError> {
        fn parse_float(span: Span, float: impl AsRef<str>) -> Result<f64, BobaError> {
            match float.as_ref().parse::<f64>() {
                Ok(float) => Ok(float),
                Err(e) => Err(BobaError::new("Parse Float Error").label(Label::new(
                    format!("{e}"),
                    Color::Red,
                    span,
                ))),
            }
        }

        fn parse_int(span: Span, int: impl AsRef<str>) -> Result<i64, BobaError> {
            match int.as_ref().parse::<i64>() {
                Ok(int) => Ok(int),
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
                    BobaError::new("Parse Integer Error").label(Label::new(
                        message,
                        Color::Red,
                        span,
                    ))
                }),
            }
        }

        match builder.next() {
            // PARENTHESES
            Some((Token::OpenParen, span)) => {
                // capture braced expression
                let braced_expr = Expr::parse_closed(builder, Some(&Token::CloseParen))?;
                // ensure expression is closed with brace
                match builder.next() {
                    Some(_) => Ok(braced_expr),
                    None => {
                        Err(
                            BobaError::new("Found unclosed parentheses while parsing expression")
                                .label(Label::new("unclosed brace found here", Color::Red, span)),
                        )
                    }
                }
            }

            // VARIABLES AND VALUES
            Some((Token::Ident(ident), _)) => Ok(Expr::Var(ident)),
            Some((Token::String(string), _)) => Ok(Expr::String(string)),
            Some((Token::Bool(bool), _)) => Ok(Expr::Bool(bool)),
            Some((Token::Int(int), span)) => Ok(Expr::Int(parse_int(span, int)?)),
            Some((Token::Float(float), span)) => Ok(Expr::Float(parse_float(span, float)?)),

            // PREFIXES
            Some((Token::Add, _)) => Self::atom_parser(builder),
            Some((Token::Not, _)) => {
                let sub_atom = builder.parse(Self::atom_parser)?;
                Ok(Expr::Not(Box::new(sub_atom)))
            }
            Some((Token::Sub, span)) => {
                match builder.peek() {
                    Some((Token::Int(int), espan)) => {
                        let int = parse_int(span.start..espan.end, format!("-{int}"))?;
                        builder.next(); // consume int token
                        Ok(Expr::Int(int))
                    }
                    Some((Token::Float(float), espan)) => {
                        let float = parse_float(span.start..espan.end, format!("-{float}"))?;
                        builder.next(); // consume float token
                        Ok(Expr::Float(float))
                    }
                    _ => {
                        let sub_atom = builder.parse(Self::atom_parser)?;
                        Ok(Expr::Neg(Box::new(sub_atom)))
                    }
                }
            }

            // ERROR FALLBACK
            Some((token, span)) => Err(BobaError::new(Self::UNEXPECTED_ERROR).label(Label::new(
                format!("Unexpected token '{token}'"),
                Color::Red,
                span,
            ))),
            None => Err(BobaError::new(
                "Found nothing while trying to parse expression",
            )),
        }
    }
}
