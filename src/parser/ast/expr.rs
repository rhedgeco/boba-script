use crate::{
    cache::CacheSpan,
    parser::{ast::Node, Lexer, PError, PResult, Token},
};

#[derive(Debug, Clone)]
pub enum Expr<Data> {
    // values
    None,
    Var(Node<String, Data>),
    Bool(Node<bool, Data>),
    Int(Node<i64, Data>),
    Float(Node<f64, Data>),
    String(Node<String, Data>),

    // function
    Call(Node<String, Data>, Vec<Node<Self, Data>>),

    // math operations
    Neg(Box<Node<Self, Data>>),
    Add(Box<Node<Self, Data>>, Box<Node<Self, Data>>),
    Sub(Box<Node<Self, Data>>, Box<Node<Self, Data>>),
    Mul(Box<Node<Self, Data>>, Box<Node<Self, Data>>),
    Div(Box<Node<Self, Data>>, Box<Node<Self, Data>>),
    Mod(Box<Node<Self, Data>>, Box<Node<Self, Data>>),
    Pow(Box<Node<Self, Data>>, Box<Node<Self, Data>>),

    // boolean operations
    Not(Box<Node<Self, Data>>),
    And(Box<Node<Self, Data>>, Box<Node<Self, Data>>),
    Or(Box<Node<Self, Data>>, Box<Node<Self, Data>>),
    Eq(Box<Node<Self, Data>>, Box<Node<Self, Data>>),
    Lt(Box<Node<Self, Data>>, Box<Node<Self, Data>>),
    Gt(Box<Node<Self, Data>>, Box<Node<Self, Data>>),
    NEq(Box<Node<Self, Data>>, Box<Node<Self, Data>>),
    LtEq(Box<Node<Self, Data>>, Box<Node<Self, Data>>),
    GtEq(Box<Node<Self, Data>>, Box<Node<Self, Data>>),

    // walrus
    Walrus(Node<String, Data>, Box<Node<Self, Data>>),

    // ternary
    Ternary(
        Box<Node<Self, Data>>,
        Box<Node<Self, Data>>,
        Box<Node<Self, Data>>,
    ),
}

impl Expr<CacheSpan> {
    fn parse_int(
        str: impl AsRef<str>,
        span: CacheSpan,
    ) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
        match str.as_ref().parse() {
            Err(error) => Err(PError::ParseIntError { error, data: span }),
            Ok(value) => Ok(Node::new(Expr::Int(Node::new(value, span)), span)),
        }
    }

    fn parse_float(
        str: impl AsRef<str>,
        span: CacheSpan,
    ) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
        match str.as_ref().parse() {
            Err(error) => Err(PError::ParseFloatError { error, data: span }),
            Ok(value) => Ok(Node::new(Expr::Float(Node::new(value, span)), span)),
        }
    }

    pub fn parse(tokens: &mut Lexer) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
        let lhs = Self::parse_atom(tokens)?;
        Self::parse_with_lhs(lhs, tokens)
    }

    pub fn parse_with_lhs(
        lhs: Node<Self, CacheSpan>,
        tokens: &mut Lexer,
    ) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
        Self::parse_walrus(lhs, tokens) // start parsing at lowest precedence operator
    }

    pub fn parse_ident(
        lhs: Node<String, CacheSpan>,
        tokens: &mut Lexer,
    ) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
        match tokens.peek() {
            Some(Err(error)) => Err(error),
            Some(Ok((Token::OpenParen, _))) => {
                tokens.next(); // consume open paren

                // check for close paren or parameters
                let mut params = Vec::new();
                match tokens.expect_peek("expression or ')'")? {
                    (Token::CloseParen, _) => (),
                    _ => loop {
                        // parse expression for parameter
                        params.push(Self::parse(tokens)?);

                        // capture comma
                        match tokens.expect_peek("',' or ')'")? {
                            (Token::Comma, _) => {
                                tokens.next(); // consume comma
                            }
                            // if no comma found, then there are no more params
                            _ => break,
                        }
                    },
                }

                // capture close paren
                let end = match tokens.expect_next("')'")? {
                    (Token::CloseParen, span) => span.range().end,
                    (token, span) => {
                        return Err(PError::UnexpectedToken {
                            expected: format!("')'"),
                            found: format!("'{token}'"),
                            data: span,
                        })
                    }
                };

                let span = tokens.span(lhs.data().range().start..end);
                Ok(Node::new(Self::Call(lhs, params), span))
            }
            Some(_) | None => {
                let span = *lhs.data();
                Ok(Node::new(Expr::Var(lhs), span))
            }
        }
    }

    pub fn parse_atom(tokens: &mut Lexer) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
        match tokens.expect_next("expression")? {
            // values
            (Token::None, span) => Ok(Node::new(Expr::None, span)),
            (Token::UInt(str), span) => Ok(Self::parse_int(str, span)?),
            (Token::UFloat(str), span) => Ok(Self::parse_float(str, span)?),
            (Token::Bool(bool), span) => Ok(Node::new(Expr::Bool(Node::new(bool, span)), span)),
            (Token::String(str), span) => {
                Ok(Node::new(Expr::String(Node::new(str.into(), span)), span))
            }

            // variables and functions
            (Token::Ident(str), span) => {
                let ident = Node::new(str.to_string(), span);
                Self::parse_ident(ident, tokens)
            }

            // prefix expressions
            (Token::Not, span) => {
                let nested = Self::parse_atom(tokens)?;
                let nested = Self::parse_powers(nested, tokens)?; // parse op with higher precedence
                let range = span.range().start..nested.data().range().end;
                Ok(Node::new(Expr::Not(Box::new(nested)), tokens.span(range)))
            }
            (Token::Sub, span) => {
                let nested = Self::parse_atom(tokens)?;
                let nested = Self::parse_powers(nested, tokens)?; // parse op with higher precedence
                let range = span.range().start..nested.data().range().end;
                Ok(Node::new(Expr::Neg(Box::new(nested)), tokens.span(range)))
            }

            // braces
            (Token::OpenParen, open_span) => {
                let inner = Self::parse(tokens)?;
                match tokens.next() {
                    Some(Err(error)) => Err(error),
                    Some(Ok((Token::CloseParen, close_span))) => Ok(Node::new(
                        inner.into_item(),
                        tokens.span(open_span.range().start..close_span.range().end),
                    )),
                    Some(Ok((_, _))) | None => Err(PError::UnclosedBrace { data: open_span }),
                }
            }

            // error case
            (token, span) => Err(PError::UnexpectedToken {
                expected: format!("expression"),
                found: format!("'{token}'"),
                data: span,
            }),
        }
    }

    pub fn parse_powers(
        lhs: Node<Self, CacheSpan>,
        tokens: &mut Lexer,
    ) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
        match tokens.peek() {
            Some(Ok((Token::Pow, _))) => (),
            Some(Err(error)) => return Err(error),
            _ => return Ok(lhs),
        }
        tokens.next();

        let rhs = Expr::parse_atom(tokens)?;
        let rhs = Self::parse_powers(rhs, tokens)?; // parse right to left

        let span = tokens.span(lhs.data().range().start..rhs.data().range().end);
        Ok(Node::new(Expr::Pow(Box::new(lhs), Box::new(rhs)), span))
    }

    pub fn parse_products(
        lhs: Node<Self, CacheSpan>,
        tokens: &mut Lexer,
    ) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
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

        let span = tokens.span(lhs.data().range().start..rhs.data().range().end);
        let new_lhs = Node::new(op(Box::new(lhs), Box::new(rhs)), span);
        Self::parse_products(new_lhs, tokens) // keep parsing
    }

    pub fn parse_sums(
        lhs: Node<Self, CacheSpan>,
        tokens: &mut Lexer,
    ) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
        let op = match tokens.peek() {
            Some(Ok((Token::Add, _))) => Expr::Add,
            Some(Ok((Token::Sub, _))) => Expr::Sub,
            Some(Err(error)) => return Err(error),
            _ => return Self::parse_products(lhs, tokens), // try next level
        };
        tokens.next();

        let rhs = Expr::parse_atom(tokens)?;
        let rhs = Self::parse_products(rhs, tokens)?; // parse higher precedence

        let span = tokens.span(lhs.data().range().start..rhs.data().range().end);
        let new_lhs = Node::new(op(Box::new(lhs), Box::new(rhs)), span);
        Self::parse_sums(new_lhs, tokens) // keep parsing
    }

    pub fn parse_comparisons(
        lhs: Node<Self, CacheSpan>,
        tokens: &mut Lexer,
    ) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
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

        let span = tokens.span(lhs.data().range().start..rhs.data().range().end);
        let new_lhs = Node::new(op(Box::new(lhs), Box::new(rhs)), span);
        Self::parse_comparisons(new_lhs, tokens) // keep parsing
    }

    pub fn parse_ands(
        lhs: Node<Self, CacheSpan>,
        tokens: &mut Lexer,
    ) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
        match tokens.peek() {
            Some(Ok((Token::And, _))) => (),
            Some(Err(error)) => return Err(error),
            _ => return Self::parse_comparisons(lhs, tokens), // try next level
        }
        tokens.next();

        let rhs = Expr::parse_atom(tokens)?;
        let rhs = Self::parse_comparisons(rhs, tokens)?; // parse higher precedence

        let span = tokens.span(lhs.data().range().start..rhs.data().range().end);
        let new_lhs = Node::new(Expr::And(Box::new(lhs), Box::new(rhs)), span);
        Self::parse_ands(new_lhs, tokens) // keep parsing
    }

    pub fn parse_ors(
        lhs: Node<Self, CacheSpan>,
        tokens: &mut Lexer,
    ) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
        match tokens.peek() {
            Some(Ok((Token::And, _))) => (),
            Some(Err(error)) => return Err(error),
            _ => return Self::parse_ands(lhs, tokens), // try next level
        }
        tokens.next();

        let rhs = Expr::parse_atom(tokens)?;
        let rhs = Self::parse_ands(rhs, tokens)?; // parse higher precedence

        let span = tokens.span(lhs.data().range().start..rhs.data().range().end);
        let new_lhs = Node::new(Expr::And(Box::new(lhs), Box::new(rhs)), span);
        Self::parse_ors(new_lhs, tokens) // keep parsing
    }

    pub fn parse_ternaries(
        lhs: Node<Self, CacheSpan>,
        tokens: &mut Lexer,
    ) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
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
                    data: span,
                })
            }
        }

        // parse false clause
        let false_clause = Expr::parse_atom(tokens)?;
        let false_clause = Self::parse_ternaries(false_clause, tokens)?; // parse right to left

        // build node
        let span = tokens.span(lhs.data().range().start..false_clause.data().range().end);
        Ok(Node::new(
            Expr::Ternary(Box::new(lhs), Box::new(true_clause), Box::new(false_clause)),
            span,
        ))
    }

    pub fn parse_walrus(
        lhs: Node<Self, CacheSpan>,
        tokens: &mut Lexer,
    ) -> PResult<Node<Self, CacheSpan>, CacheSpan> {
        let walrus_span = match tokens.peek() {
            Some(Ok((Token::Walrus, span))) => span.clone(),
            Some(Err(error)) => return Err(error),
            _ => return Self::parse_ternaries(lhs, tokens), // try next level
        };
        tokens.next();

        let lhs = match lhs.into_parts() {
            (Expr::Var(var), _) => var,
            (_, _) => return Err(PError::InvalidWalrusAssignment { data: walrus_span }),
        };

        let rhs = Self::parse_atom(tokens)?;
        let rhs = Self::parse_walrus(rhs, tokens)?; // parse right to left

        let span = tokens.span(lhs.data().range().start..rhs.data().range().end);
        Ok(Node::new(Expr::Walrus(lhs, Box::new(rhs)), span))
    }
}
