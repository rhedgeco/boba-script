use std::num::{ParseFloatError, ParseIntError};

use derive_more::Display;
use logos::Logos;
use once_cell::sync::Lazy;
use regex::Regex;

pub type Span = logos::Span;

#[derive(Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[display(fmt = "{}", _0)]
pub struct Ident(String);
impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Ident {
    pub fn new(ident: impl Into<String>) -> Option<Self> {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^[_a-zA-Z][_a-zA-z0-9]*$").expect("Invalid Ident regex"));

        let ident = ident.into();
        REGEX.is_match(&ident).then(|| Self(ident))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum TokenError {
    #[default]
    UnexpectedToken,
    ParseIntError(ParseIntError),
    ParseFloatError,
}

impl From<ParseIntError> for TokenError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseIntError(value)
    }
}

impl From<ParseFloatError> for TokenError {
    fn from(_: ParseFloatError) -> Self {
        Self::ParseFloatError
    }
}

impl TokenError {
    pub fn get_message(&self) -> String {
        match self {
            Self::UnexpectedToken => format!("Unexpected token"),
            Self::ParseIntError(e) => match e.kind() {
                std::num::IntErrorKind::PosOverflow => {
                    format!("Integer is too large. Must be less than 9,223,372,036,854,775,807")
                }
                _ => format!("Unknown integer error :("),
            },
            Self::ParseFloatError => format!("Invalid float literal"),
        }
    }
}

fn remove_quotes(str: impl AsRef<str>) -> String {
    let str = str.as_ref();
    match str.strip_prefix("'") {
        Some(stripped) => stripped.strip_suffix("'").unwrap_or(str),
        None => match str.strip_prefix("\"") {
            Some(stripped) => stripped.strip_suffix("\"").unwrap_or(str),
            None => str,
        },
    }
    .to_string()
}

#[derive(Logos, Debug, Clone, PartialEq, PartialOrd)]
#[logos(skip r"[ \t\n\r\f]")] // skip whitespace
#[logos(error = TokenError)]
pub enum Token {
    #[regex(r"[_a-zA-Z][_a-zA-z0-9]*", |lex| Ident::new(lex.slice()))]
    Ident(Ident),

    // numbers
    #[regex(r"[0-9]+", |lex| lex.slice().parse())]
    Int(i64),
    #[regex(r"[0-9]*\.[0-9]+", |lex| lex.slice().parse())]
    Float(f64),
    #[regex("('[^']*')|(\"[^\"]*\")", |lex| remove_quotes(lex.slice()))]
    String(String),

    // operators
    #[token("=")]
    Equal,
    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("**")]
    Pow,

    // keywords
    #[token("let")]
    Let,

    // control flow
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
}