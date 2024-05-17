use crate::{
    parser::{report::PError, TokenSource},
    token::Span,
    Token,
};

use super::{
    span::Spanner,
    values::{Bool, Float, Int, StringSpan},
    Ident, Spanned,
};

#[derive(Debug)]
pub enum Expr {
    // values
    Unit(Span),
    Var(Ident),
    Int(Int),
    Float(Float),
    Bool(Bool),
    String(StringSpan),

    // math operators
    Neg(Spanner<Box<Self>>),
    Pow(Box<Self>, Box<Self>),
    Mul(Box<Self>, Box<Self>),
    Div(Box<Self>, Box<Self>),
    Mod(Box<Self>, Box<Self>),
    Add(Box<Self>, Box<Self>),
    Sub(Box<Self>, Box<Self>),

    // boolean operators
    Not(Spanner<Box<Self>>),
    Eq(Box<Self>, Box<Self>),
    Lt(Box<Self>, Box<Self>),
    Gt(Box<Self>, Box<Self>),
    NEq(Box<Self>, Box<Self>),
    LtEq(Box<Self>, Box<Self>),
    GtEq(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    And(Box<Self>, Box<Self>),

    // assignment
    Assign(Ident, Box<Self>),
    Walrus(Ident, Box<Self>),

    // ternary
    Ternary(Box<Self>, Box<Self>, Box<Self>),
}

impl Spanned for Expr {
    fn span(&self) -> Span {
        match self {
            Self::Unit(span) => span.clone(),
            Self::Var(ident) => ident.span(),
            Self::Int(int) => int.span(),
            Self::Float(float) => float.span(),
            Self::Bool(bool) => bool.span(),
            Self::String(string) => string.span(),
            Self::Neg(expr) => expr.span(),
            Self::Not(expr) => expr.span(),
            Self::Pow(e1, e2) => e1.span().start..e2.span().end,
            Self::Mul(e1, e2) => e1.span().start..e2.span().end,
            Self::Div(e1, e2) => e1.span().start..e2.span().end,
            Self::Mod(e1, e2) => e1.span().start..e2.span().end,
            Self::Add(e1, e2) => e1.span().start..e2.span().end,
            Self::Sub(e1, e2) => e1.span().start..e2.span().end,
            Self::Eq(e1, e2) => e1.span().start..e2.span().end,
            Self::Lt(e1, e2) => e1.span().start..e2.span().end,
            Self::Gt(e1, e2) => e1.span().start..e2.span().end,
            Self::NEq(e1, e2) => e1.span().start..e2.span().end,
            Self::LtEq(e1, e2) => e1.span().start..e2.span().end,
            Self::GtEq(e1, e2) => e1.span().start..e2.span().end,
            Self::Or(e1, e2) => e1.span().start..e2.span().end,
            Self::And(e1, e2) => e1.span().start..e2.span().end,
            Self::Ternary(e1, _, e2) => e1.span().start..e2.span().end,
            Self::Assign(i, e) => i.span().start..e.span().end,
            Self::Walrus(i, e) => i.span().start..e.span().end,
        }
    }
}

impl Expr {
    pub fn parse_unit(source: &mut TokenSource) -> Result<Self, PError> {
        match source.take() {
            Some((Token::Unit, span)) => Ok(Self::Unit(span)),
            Some((token, span)) => Err(PError::UnexpectedToken {
                expect: "unit".into(),
                found: format!("'{token}'"),
                span,
            }),
            None => Err(PError::UnexpectedEnd {
                expect: "unit".into(),
                pos: source.pos(),
            }),
        }
    }

    pub fn parse_atom(source: &mut TokenSource) -> Result<Self, PError> {
        match source.peek() {
            // PARSE VALUES
            Some((Token::Unit, _)) => Self::parse_unit(source),
            Some((Token::Ident(_), _)) => Ok(Self::Var(Ident::parse(source)?)),
            Some((Token::Int(_), _)) => Ok(Self::Int(Int::parse(source)?)),
            Some((Token::Float(_), _)) => Ok(Self::Float(Float::parse(source)?)),
            Some((Token::Bool(_), _)) => Ok(Self::Bool(Bool::parse(source)?)),
            Some((Token::String(_), _)) => Ok(Self::String(StringSpan::parse(source)?)),

            // PARSE BOOLEAN NEGATION
            Some((Token::Bang, span)) => {
                let span_start = span.start;
                let nested = Self::parse_atom(source)?;
                Ok(Expr::Not(Spanner::new(
                    span_start..nested.span().end,
                    Box::new(nested),
                )))
            }

            // PARSE NEGATIVES
            Some((Token::Sub, sub_span)) => {
                let span_start = sub_span.start;
                source.take(); // consume sub token
                match source.peek() {
                    Some((Token::Int(str), span)) => {
                        let nested = Int::parse_str(format!("-{str}"), span_start..span.end)?;
                        Ok(Self::Int(nested))
                    }
                    Some((Token::Float(str), span)) => {
                        let nested = Float::parse_str(format!("-{str}"), span_start..span.end)?;
                        Ok(Self::Float(nested))
                    }
                    Some(_) => {
                        let nested = Self::parse_atom(source)?;
                        Ok(Self::Neg(Spanner::new(
                            span_start..nested.span().end,
                            Box::new(nested),
                        )))
                    }
                    None => Err(PError::UnexpectedEnd {
                        expect: "expression".into(),
                        pos: source.pos(),
                    }),
                }
            }

            // PARSE BRACED EXPRESSIONS
            Some((Token::OpenParen, open_span)) => {
                let open_span = open_span.clone();
                source.take(); // consume open token
                let inner_expr = Self::parse_until(source, |t| t == &Token::CloseParen)?;
                match source.peek() {
                    Some((Token::CloseParen, _)) => {
                        source.take(); // consume close token
                        Ok(inner_expr)
                    }
                    Some((_, _)) => unreachable!(),
                    None => Err(PError::UnclosedBrace {
                        open_span: open_span.clone(),
                        close_message: "reached end with no closing brace".into(),
                        close_span: source.pos()..source.pos(),
                    }),
                }
            }

            // ERROR CASES
            Some((token, span)) => Err(PError::UnexpectedToken {
                expect: "expression".into(),
                found: format!("'{token}'"),
                span: span.clone(),
            }),
            None => Err(PError::UnexpectedEnd {
                expect: "expression".into(),
                pos: source.pos(),
            }),
        }
    }

    /// Parses the provided [`TokenSource`] as an [`Expr`] until the end.
    ///
    /// Equivilant to calling [`Expr::parse_until`] using `|_| false`.
    pub fn parse(source: &mut TokenSource) -> Result<Self, PError> {
        Self::parse_until(source, |_| false)
    }

    /// Parses the provided [`TokenSource`] as an [`Expr`] starting with `lhs`.
    ///
    /// Stops when `until` evaluates to `true`.
    /// The `until` function is only run when an unexpected token is found.
    /// So if `until` expects a token that is used as an operator, it will not evaluate.
    ///
    /// EXAMPLE: `Token::Colon` will trigger the `until` evaluation,
    /// but `Token::Add` will not since it will be used as an operator in the expression.
    pub fn parse_until(
        source: &mut TokenSource,
        until: impl Fn(&Token) -> bool,
    ) -> Result<Self, PError> {
        fn try_parse_pow(lhs: Expr, source: &mut TokenSource) -> Result<Expr, PError> {
            match source.peek() {
                Some((Token::Pow, _)) => (),
                _ => return Ok(lhs),
            };

            source.take(); // consume token
            let rhs = Expr::parse_atom(source)?;
            Ok(Expr::Pow(Box::new(lhs), Box::new(rhs)))
        }

        fn try_parse_mul(lhs: Expr, source: &mut TokenSource) -> Result<Expr, PError> {
            let op = match source.peek() {
                Some((Token::Mul, _)) => Expr::Mul,
                Some((Token::Div, _)) => Expr::Div,
                Some((Token::Mod, _)) => Expr::Mod,
                _ => return try_parse_pow(lhs, source),
            };

            source.take(); // consume token
            let rhs = Expr::parse_atom(source)?;
            let rhs = try_parse_pow(rhs, source)?; // ensure op precedence
            Ok(op(Box::new(lhs), Box::new(rhs)))
        }

        fn try_parse_add(lhs: Expr, source: &mut TokenSource) -> Result<Expr, PError> {
            let op = match source.peek() {
                Some((Token::Add, _)) => Expr::Add,
                Some((Token::Sub, _)) => Expr::Sub,
                _ => return try_parse_mul(lhs, source),
            };

            source.take(); // consume token
            let rhs = Expr::parse_atom(source)?;
            let rhs = try_parse_mul(rhs, source)?; // ensure op precedence
            Ok(op(Box::new(lhs), Box::new(rhs)))
        }

        fn try_parse_bool(lhs: Expr, source: &mut TokenSource) -> Result<Expr, PError> {
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
            Ok(op(Box::new(lhs), Box::new(rhs)))
        }

        fn try_parse_and(lhs: Expr, source: &mut TokenSource) -> Result<Expr, PError> {
            let op = match source.peek() {
                Some((Token::And, _)) => Expr::And,
                _ => return try_parse_bool(lhs, source),
            };

            source.take(); // consume token
            let rhs = Expr::parse_atom(source)?;
            let rhs = try_parse_bool(rhs, source)?; // ensure op precedence
            Ok(op(Box::new(lhs), Box::new(rhs)))
        }

        fn try_parse_or(lhs: Expr, source: &mut TokenSource) -> Result<Expr, PError> {
            let op = match source.peek() {
                Some((Token::Or, _)) => Expr::Or,
                _ => return try_parse_and(lhs, source),
            };

            source.take(); // consume token
            let rhs = Expr::parse_atom(source)?;
            let rhs = try_parse_and(rhs, source)?; // ensure op precedence
            Ok(op(Box::new(lhs), Box::new(rhs)))
        }

        fn try_parse_ternary(lhs: Expr, source: &mut TokenSource) -> Result<Expr, PError> {
            let if_span = match source.peek() {
                Some((Token::If, span)) => span.clone(),
                _ => return try_parse_and(lhs, source),
            };

            // consume if
            source.take();

            // parse condition
            let cond = match Expr::parse_until(source, |t| t == &Token::Else) {
                Ok(cond) => cond,
                // make unexpected token error clearer
                Err(PError::UnexpectedToken {
                    expect: _,
                    found,
                    span,
                }) => {
                    return Err(PError::UnexpectedToken {
                        expect: "operator or 'else'".into(),
                        found,
                        span,
                    })
                }
                Err(e) => return Err(e),
            };

            // ensure 'else' exists
            match source.peek() {
                Some((Token::Else, _)) => source.take(),
                Some((_, _)) => unreachable!(),
                None => {
                    return Err(PError::IncompleteTernary {
                        if_span,
                        end: source.pos(),
                    })
                }
            };

            // parse else expression until end
            let rhs = Expr::parse(source)?;

            // construct ternary
            Ok(Expr::Ternary(Box::new(lhs), Box::new(cond), Box::new(rhs)))
        }

        fn try_parse_assign(lhs: Expr, source: &mut TokenSource) -> Result<Expr, PError> {
            let op = match source.peek() {
                Some((Token::Assign, _)) => Expr::Assign,
                Some((Token::Walrus, _)) => Expr::Walrus,
                _ => return try_parse_ternary(lhs, source),
            };

            let ident = match lhs {
                Expr::Var(ident) => ident,
                _ => {
                    return Err(PError::AssignmentError {
                        lhs_span: lhs.span(),
                    })
                }
            };

            source.take(); // consume token
            let rhs = Expr::parse_atom(source)?;
            let rhs = try_parse_ternary(rhs, source)?; // ensure op precedence
            Ok(op(ident, Box::new(rhs)))
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

                // parse ternary
                Some((Token::If, _)) => try_parse_ternary(lhs, source)?,

                // parse assignment
                Some((Token::Assign, _)) | Some((Token::Walrus, _)) => {
                    try_parse_assign(lhs, source)?
                }

                // check for failure or ending token
                Some((token, span)) => match until(token) {
                    true => return Ok(lhs),
                    false => {
                        let span = span.clone();
                        let token = token.clone();
                        source.take(); // consume error token
                        return Err(PError::UnexpectedToken {
                            expect: "operator".into(),
                            found: format!("'{token}'"),
                            span: span.clone(),
                        });
                    }
                },
            };
        }
    }
}
