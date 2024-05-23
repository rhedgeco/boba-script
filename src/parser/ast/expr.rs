use crate::parser::{ast::Node, PError, PResult, Span, Token, TokenLine};

#[derive(Debug, Clone)]
pub enum Expr {
    // values
    None,
    Var(String),
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),

    // math operations
    Neg(Box<Node<Expr>>),
    Add(Box<Node<Expr>>, Box<Node<Expr>>),
    Sub(Box<Node<Expr>>, Box<Node<Expr>>),
    Mul(Box<Node<Expr>>, Box<Node<Expr>>),
    Div(Box<Node<Expr>>, Box<Node<Expr>>),
    Mod(Box<Node<Expr>>, Box<Node<Expr>>),
    Pow(Box<Node<Expr>>, Box<Node<Expr>>),

    // boolean operations
    Not(Box<Node<Expr>>),
    And(Box<Node<Expr>>, Box<Node<Expr>>),
    Or(Box<Node<Expr>>, Box<Node<Expr>>),
    Eq(Box<Node<Expr>>, Box<Node<Expr>>),
    Lt(Box<Node<Expr>>, Box<Node<Expr>>),
    Gt(Box<Node<Expr>>, Box<Node<Expr>>),
    NEq(Box<Node<Expr>>, Box<Node<Expr>>),
    LtEq(Box<Node<Expr>>, Box<Node<Expr>>),
    GtEq(Box<Node<Expr>>, Box<Node<Expr>>),

    // walrus
    Walrus(Node<String>, Box<Node<Expr>>),

    // ternary
    Ternary(Box<Node<Expr>>, Box<Node<Expr>>, Box<Node<Expr>>),
}

impl Expr {
    fn parse_int(span: Span, str: impl AsRef<str>) -> PResult<Node<Expr>> {
        match str.as_ref().parse() {
            Err(error) => Err(PError::ParseIntError { error, span }),
            Ok(value) => Ok(Node::new(span, Expr::Int(value))),
        }
    }

    fn parse_float(span: Span, str: impl AsRef<str>) -> PResult<Node<Expr>> {
        match str.as_ref().parse() {
            Err(error) => Err(PError::ParseFloatError { error, span }),
            Ok(value) => Ok(Node::new(span, Expr::Float(value))),
        }
    }

    pub fn parse(tokens: &mut TokenLine) -> PResult<Node<Self>> {
        let lhs = Self::parse_atom(tokens)?;
        Self::parse_with(lhs, tokens)
    }

    pub fn parse_with(lhs: Node<Expr>, tokens: &mut TokenLine) -> PResult<Node<Self>> {
        Self::parse_walrus(lhs, tokens) // start parsing at lowest precedence operator
    }

    pub fn parse_atom(tokens: &mut TokenLine) -> PResult<Node<Self>> {
        match tokens.expect_next("expression")? {
            // values
            (Token::None, span) => Ok(Node::new(span, Expr::None)),
            (Token::Ident(str), span) => Ok(Node::new(span, Expr::Var(str.into()))),
            (Token::Bool(bool), span) => Ok(Node::new(span, Expr::Bool(bool))),
            (Token::UInt(str), span) => Ok(Self::parse_int(span, str)?),
            (Token::UFloat(str), span) => Ok(Self::parse_float(span, str)?),
            (Token::String(str), span) => Ok(Node::new(span, Expr::String(str.into()))),

            // prefix expressions
            (Token::Not, span) => {
                let nested = Self::parse_atom(tokens)?;
                let nested = Self::parse_powers(nested, tokens)?; // parse op with higher precedence
                let span = span.start..nested.span().end;
                Ok(Node::new(span, Expr::Not(Box::new(nested))))
            }
            (Token::Sub, span) => {
                let nested = Self::parse_atom(tokens)?;
                let nested = Self::parse_powers(nested, tokens)?; // parse op with higher precedence
                let span = span.start..nested.span().end;
                Ok(Node::new(span, Expr::Neg(Box::new(nested))))
            }

            // braces
            (Token::OpenParen, open_span) => {
                let inner = Self::parse(tokens)?;
                match tokens.next() {
                    Some(Err(error)) => Err(error),
                    Some(Ok((Token::CloseParen, close_span))) => Ok(Node::new(
                        open_span.start..close_span.end,
                        inner.into_item(),
                    )),
                    Some(Ok((token, span))) => Err(PError::UnclosedBrace {
                        found: format!("'{token}'"),
                        open: open_span,
                        close: span,
                    }),
                    None => Err(PError::UnclosedBrace {
                        found: format!("end of line"),
                        open: open_span,
                        close: tokens.line_end()..tokens.line_end(),
                    }),
                }
            }

            // error case
            (token, span) => Err(PError::UnexpectedToken {
                expected: format!("expression"),
                found: format!("'{token}'"),
                span,
            }),
        }
    }

    pub fn parse_powers(lhs: Node<Expr>, tokens: &mut TokenLine) -> PResult<Node<Expr>> {
        match tokens.peek() {
            Some(Ok((Token::Pow, _))) => (),
            Some(Err(error)) => return Err(error),
            _ => return Ok(lhs),
        }
        tokens.next();

        let rhs = Expr::parse_atom(tokens)?;
        let rhs = Self::parse_powers(rhs, tokens)?; // parse right to left

        let span = lhs.span().start..rhs.span().end;
        Ok(Node::new(span, Expr::Pow(Box::new(lhs), Box::new(rhs))))
    }

    pub fn parse_products(lhs: Node<Expr>, tokens: &mut TokenLine) -> PResult<Node<Expr>> {
        let op = match tokens.peek() {
            Some(Ok((Token::Mul, _))) => Expr::Mul,
            Some(Ok((Token::Div, _))) => Expr::Div,
            Some(Ok((Token::Mod, _))) => Expr::Mod,
            Some(Err(error)) => return Err(error),
            _ => return Self::parse_powers(lhs, tokens), // try next level
        };
        tokens.next();

        let rhs = Expr::parse_atom(tokens)?;
        let rhs = Self::parse_powers(rhs, tokens)?; // parse higher precedence

        let span = lhs.span().start..rhs.span().end;
        let new_lhs = Node::new(span, op(Box::new(lhs), Box::new(rhs)));
        Self::parse_products(new_lhs, tokens) // keep parsing
    }

    pub fn parse_sums(lhs: Node<Expr>, tokens: &mut TokenLine) -> PResult<Node<Expr>> {
        let op = match tokens.peek() {
            Some(Ok((Token::Add, _))) => Expr::Add,
            Some(Ok((Token::Sub, _))) => Expr::Sub,
            Some(Err(error)) => return Err(error),
            _ => return Self::parse_products(lhs, tokens), // try next level
        };
        tokens.next();

        let rhs = Expr::parse_atom(tokens)?;
        let rhs = Self::parse_products(rhs, tokens)?; // parse higher precedence

        let span = lhs.span().start..rhs.span().end;
        let new_lhs = Node::new(span, op(Box::new(lhs), Box::new(rhs)));
        Self::parse_sums(new_lhs, tokens) // keep parsing
    }

    pub fn parse_comparisons(lhs: Node<Expr>, tokens: &mut TokenLine) -> PResult<Node<Expr>> {
        let op = match tokens.peek() {
            Some(Ok((Token::Eq, _))) => Expr::Eq,
            Some(Ok((Token::Lt, _))) => Expr::Lt,
            Some(Ok((Token::Gt, _))) => Expr::Gt,
            Some(Ok((Token::NEq, _))) => Expr::NEq,
            Some(Ok((Token::LtEq, _))) => Expr::LtEq,
            Some(Ok((Token::GtEq, _))) => Expr::GtEq,
            Some(Err(error)) => return Err(error),
            _ => return Self::parse_sums(lhs, tokens), // try next level
        };
        tokens.next();

        let rhs = Expr::parse_atom(tokens)?;
        let rhs = Self::parse_sums(rhs, tokens)?; // parse higher precedence

        let span = lhs.span().start..rhs.span().end;
        let new_lhs = Node::new(span, op(Box::new(lhs), Box::new(rhs)));
        Self::parse_comparisons(new_lhs, tokens) // keep parsing
    }

    pub fn parse_ands(lhs: Node<Expr>, tokens: &mut TokenLine) -> PResult<Node<Expr>> {
        match tokens.peek() {
            Some(Ok((Token::And, _))) => (),
            Some(Err(error)) => return Err(error),
            _ => return Self::parse_comparisons(lhs, tokens), // try next level
        }
        tokens.next();

        let rhs = Expr::parse_atom(tokens)?;
        let rhs = Self::parse_comparisons(rhs, tokens)?; // parse higher precedence

        let span = lhs.span().start..rhs.span().end;
        let new_lhs = Node::new(span, Expr::And(Box::new(lhs), Box::new(rhs)));
        Self::parse_ands(new_lhs, tokens) // keep parsing
    }

    pub fn parse_ors(lhs: Node<Expr>, tokens: &mut TokenLine) -> PResult<Node<Expr>> {
        match tokens.peek() {
            Some(Ok((Token::And, _))) => (),
            Some(Err(error)) => return Err(error),
            _ => return Self::parse_ands(lhs, tokens), // try next level
        }
        tokens.next();

        let rhs = Expr::parse_atom(tokens)?;
        let rhs = Self::parse_ands(rhs, tokens)?; // parse higher precedence

        let span = lhs.span().start..rhs.span().end;
        let new_lhs = Node::new(span, Expr::And(Box::new(lhs), Box::new(rhs)));
        Self::parse_ors(new_lhs, tokens) // keep parsing
    }

    pub fn parse_ternaries(lhs: Node<Expr>, tokens: &mut TokenLine) -> PResult<Node<Expr>> {
        // check for question delimiter
        match tokens.peek() {
            Some(Ok((Token::Question, _))) => (),
            Some(Err(error)) => return Err(error),
            _ => return Self::parse_ors(lhs, tokens), // try next level
        }
        tokens.next();

        // parse true clause
        let true_clause = Expr::parse_atom(tokens)?;
        let true_clause = Self::parse_ternaries(true_clause, tokens)?; // parse right to left

        // check for colon delimiter
        match tokens.expect_next("ternary delimiter ':'")? {
            (Token::Colon, _) => (),
            (token, span) => {
                return Err(PError::UnexpectedToken {
                    expected: format!("ternary delimiter ':'"),
                    found: format!("'{token}'"),
                    span,
                })
            }
        }

        // parse false clause
        let false_clause = Expr::parse_atom(tokens)?;
        let false_clause = Self::parse_ternaries(false_clause, tokens)?; // parse right to left

        // build node
        Ok(Node::new(
            lhs.span().start..false_clause.span().end,
            Expr::Ternary(Box::new(lhs), Box::new(true_clause), Box::new(false_clause)),
        ))
    }

    pub fn parse_walrus(lhs: Node<Expr>, tokens: &mut TokenLine) -> PResult<Node<Expr>> {
        let walrus_span = match tokens.peek() {
            Some(Ok((Token::Walrus, span))) => span.clone(),
            Some(Err(error)) => return Err(error),
            _ => return Self::parse_ternaries(lhs, tokens), // try next level
        };
        tokens.next();

        let lhs = match lhs.into_parts() {
            (span, Expr::Var(var)) => Node::new(span, var),
            (expr_span, _) => {
                return Err(PError::InvalidWalrusAssignment {
                    walrus_span,
                    expr_span,
                })
            }
        };

        let rhs = Self::parse_atom(tokens)?;
        let rhs = Self::parse_walrus(rhs, tokens)?; // parse right to left

        let span = lhs.span().start..rhs.span().end;
        Ok(Node::new(span, Expr::Walrus(lhs, Box::new(rhs))))
    }
}
