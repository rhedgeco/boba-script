use crate::{
    lexer::Token,
    parser::{
        node::NodeBuilderExt,
        report::{PError, PResult},
        Node, TokenSource,
    },
};

#[derive(Debug)]
pub enum Expr {
    // values
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),

    // math operators
    Neg(Box<Node<Expr>>),
    Pow(Box<Node<Expr>>, Box<Node<Expr>>),
    Mul(Box<Node<Expr>>, Box<Node<Expr>>),
    Div(Box<Node<Expr>>, Box<Node<Expr>>),
    Mod(Box<Node<Expr>>, Box<Node<Expr>>),
    Add(Box<Node<Expr>>, Box<Node<Expr>>),
    Sub(Box<Node<Expr>>, Box<Node<Expr>>),

    // boolean operators
    Not(Box<Node<Expr>>),
    Eq(Box<Node<Expr>>, Box<Node<Expr>>),
    Lt(Box<Node<Expr>>, Box<Node<Expr>>),
    Gt(Box<Node<Expr>>, Box<Node<Expr>>),
    NEq(Box<Node<Expr>>, Box<Node<Expr>>),
    LtEq(Box<Node<Expr>>, Box<Node<Expr>>),
    GtEq(Box<Node<Expr>>, Box<Node<Expr>>),
    Or(Box<Node<Expr>>, Box<Node<Expr>>),
    And(Box<Node<Expr>>, Box<Node<Expr>>),
}

impl Expr {
    pub fn parse_atom<'a>(source: &mut impl TokenSource<'a>) -> PResult<Node<Self>> {
        let mut builder = source.node_builder();
        match builder.take() {
            // PARSE INTEGERS
            Some((Token::Int(str), span)) => match str.parse() {
                Ok(value) => Ok(builder.build(Expr::Int(value))),
                Err(error) => Err(PError::ParseIntError { error, span }.into()),
            },

            // PARSE FLOATS
            Some((Token::Float(str), span)) => match str.parse() {
                Ok(value) => Ok(builder.build(Expr::Float(value))),
                Err(error) => Err(PError::ParseFloatError { error, span }.into()),
            },

            // PARSE BOOLS
            Some((Token::Bool(bool), _)) => Ok(builder.build(Expr::Bool(bool))),

            // PARSE STRINGS
            Some((Token::String(str), _)) => Ok(builder.build(Expr::String(str.to_string()))),

            // PARSE NEGATIVES
            Some((Token::Sub, sub_span)) => match builder.peek() {
                Some((Token::Int(str), span)) => match format!("-{str}").parse() {
                    Ok(value) => Ok(builder.build(Expr::Int(value))),
                    Err(error) => Err(PError::ParseIntError {
                        error,
                        span: sub_span.start..span.end,
                    }
                    .into()),
                },
                Some((Token::Float(str), span)) => match format!("-{str}").parse() {
                    Ok(value) => Ok(builder.build(Expr::Float(value))),
                    Err(error) => Err(PError::ParseFloatError {
                        error,
                        span: sub_span.start..span.end,
                    }
                    .into()),
                },
                Some(_) => {
                    let nested = Self::parse_atom(&mut builder)?;
                    Ok(builder.build(Expr::Neg(Box::new(nested))))
                }
                None => Err(PError::UnexpectedEnd {
                    expect: "expression".into(),
                    pos: builder.pos(),
                }
                .into()),
            },

            // PARSE BOOLEAN NEGATION
            Some((Token::Bang, _)) => {
                // bang notation applies not to a single atom
                let nested = Self::parse_atom(&mut builder)?;
                Ok(builder.build(Expr::Not(Box::new(nested))))
            }
            Some((Token::Not, _)) => {
                // logical not has lower priority and captures the whole expression
                let nested = Self::parse(&mut builder)?;
                Ok(builder.build(Expr::Not(Box::new(nested))))
            }

            // PARSE BRACED EXPRESSIONS
            Some((Token::OpenParen, open_span)) => {
                let inner_expr = Self::parse_until(&mut builder, |t| t == &Token::CloseParen)?;
                match builder.peek() {
                    Some((Token::CloseParen, _)) => {
                        builder.take(); // consume close paren
                        Ok(inner_expr)
                    }
                    Some((_, _)) => unreachable!(),
                    None => Err(PError::UnclosedBrace {
                        open_span,
                        close_message: "reached end with no closing brace".into(),
                        close_span: builder.pos()..builder.pos(),
                    }
                    .into()),
                }
            }

            // ERROR CASES
            Some((token, span)) => Err(PError::UnexpectedToken {
                expect: "expression".into(),
                found: format!("'{token}'"),
                span,
            }
            .into()),
            None => Err(PError::UnexpectedEnd {
                expect: "expression".into(),
                pos: source.pos(),
            }
            .into()),
        }
    }

    /// Parses the provided [`TokenSource`] as an [`Expr`] until the end
    ///
    /// Equivilant to calling [`Expr::parse_until`] using `|_| false`.
    pub fn parse<'a>(source: &mut impl TokenSource<'a>) -> PResult<Node<Self>> {
        Self::parse_until(source, |_| false)
    }

    /// Parses the provided [`TokenSource`] as an [`Expr`]
    ///
    /// Stops when `until` evaluates to `true`.
    /// The `until` function is only run when an unexpected token is found.
    /// So if `until` expects a token that is used as an operator, it will not evaluate.
    ///
    /// EXAMPLE: `Token::Colon` will trigger the `until` evaluation,
    /// but `Token::Add` will not since it will be used as an operator in the expression.
    pub fn parse_until<'a>(
        source: &mut impl TokenSource<'a>,
        until: impl Fn(&Token) -> bool,
    ) -> PResult<Node<Self>> {
        fn try_parse_pow<'a>(
            lhs: Node<Expr>,
            source: &mut impl TokenSource<'a>,
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
            source: &mut impl TokenSource<'a>,
        ) -> PResult<Node<Expr>> {
            let op = match source.peek() {
                Some((Token::Mul, _)) => Expr::Mul,
                Some((Token::Div, _)) => Expr::Div,
                Some((Token::Mod, _)) => Expr::Mod,
                _ => return try_parse_pow(lhs, source),
            };

            source.take(); // consume token
            let rhs = Expr::parse_atom(source)?;
            let rhs = try_parse_pow(rhs, source)?; // ensure op precedence
            let span = lhs.span().start..rhs.span().end;
            Ok(Node::build(span, op(Box::new(lhs), Box::new(rhs))))
        }

        fn try_parse_add<'a>(
            lhs: Node<Expr>,
            source: &mut impl TokenSource<'a>,
        ) -> PResult<Node<Expr>> {
            let op = match source.peek() {
                Some((Token::Add, _)) => Expr::Add,
                Some((Token::Sub, _)) => Expr::Sub,
                _ => return try_parse_mul(lhs, source),
            };

            source.take(); // consume token
            let rhs = Expr::parse_atom(source)?;
            let rhs = try_parse_mul(rhs, source)?; // ensure op precedence
            let span = lhs.span().start..rhs.span().end;
            Ok(Node::build(span, op(Box::new(lhs), Box::new(rhs))))
        }

        fn try_parse_bool<'a>(
            lhs: Node<Expr>,
            source: &mut impl TokenSource<'a>,
        ) -> PResult<Node<Expr>> {
            let op = match source.peek() {
                Some((Token::Eq, _)) => Expr::Eq,
                Some((Token::Lt, _)) => Expr::Lt,
                Some((Token::Gt, _)) => Expr::Gt,
                Some((Token::NEq, _)) => Expr::NEq,
                Some((Token::LtEq, _)) => Expr::LtEq,
                Some((Token::GtEq, _)) => Expr::GtEq,
                _ => return try_parse_add(lhs, source),
            };

            source.take(); // consume token
            let rhs = Expr::parse_atom(source)?;
            let rhs = try_parse_add(rhs, source)?; // ensure op precedence
            let span = lhs.span().start..rhs.span().end;
            Ok(Node::build(span, op(Box::new(lhs), Box::new(rhs))))
        }

        fn try_parse_and<'a>(
            lhs: Node<Expr>,
            source: &mut impl TokenSource<'a>,
        ) -> PResult<Node<Expr>> {
            let op = match source.peek() {
                Some((Token::And, _)) => Expr::And,
                _ => return try_parse_bool(lhs, source),
            };

            source.take(); // consume token
            let rhs = Expr::parse_atom(source)?;
            let rhs = try_parse_bool(rhs, source)?; // ensure op precedence
            let span = lhs.span().start..rhs.span().end;
            Ok(Node::build(span, op(Box::new(lhs), Box::new(rhs))))
        }

        fn try_parse_or<'a>(
            lhs: Node<Expr>,
            source: &mut impl TokenSource<'a>,
        ) -> PResult<Node<Expr>> {
            let op = match source.peek() {
                Some((Token::Or, _)) => Expr::Or,
                _ => return try_parse_and(lhs, source),
            };

            source.take(); // consume token
            let rhs = Expr::parse_atom(source)?;
            let rhs = try_parse_and(rhs, source)?; // ensure op precedence
            let span = lhs.span().start..rhs.span().end;
            Ok(Node::build(span, op(Box::new(lhs), Box::new(rhs))))
        }

        // parse initial atom
        let mut lhs = Self::parse_atom(source)?;

        // loop until all ops are handled
        loop {
            lhs = match source.peek() {
                // return after no tokens
                None => return Ok(lhs),

                // parse power
                Some((Token::Pow, _)) => try_parse_pow(lhs, source)?,

                // parse add/sub
                Some((Token::Add, _)) | Some((Token::Sub, _)) => try_parse_add(lhs, source)?,

                // parse mul/div/mod
                Some((Token::Mul, _)) | Some((Token::Div, _)) | Some((Token::Mod, _)) => {
                    try_parse_mul(lhs, source)?
                }

                // parse relational
                Some((Token::Eq, _))
                | Some((Token::Lt, _))
                | Some((Token::Gt, _))
                | Some((Token::NEq, _))
                | Some((Token::LtEq, _))
                | Some((Token::GtEq, _)) => try_parse_bool(lhs, source)?,

                // parse and
                Some((Token::And, _)) => try_parse_and(lhs, source)?,

                // parse or
                Some((Token::Or, _)) => try_parse_or(lhs, source)?,

                // check for failure or ending token
                Some((token, span)) => match until(token) {
                    true => return Ok(lhs),
                    false => {
                        return Err(PError::UnexpectedToken {
                            expect: "operator".into(),
                            found: format!("'{token}'"),
                            span: span.clone(),
                        }
                        .into())
                    }
                },
            };
        }
    }
}
