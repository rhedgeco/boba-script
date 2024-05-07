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
pub enum Token {
    #[regex(r"[_a-zA-Z][_a-zA-z0-9]*", |lex| Ident::new(lex.slice()))]
    Ident(Ident),

    // values
    #[regex(r"true|false", |lex| lex.slice() == "true")]
    Bool(bool),
    #[regex(r"[0-9]+", |lex| lex.slice().to_string())]
    Int(String),
    #[regex(r"[0-9]*\.[0-9]+", |lex| lex.slice().to_string())]
    Float(String),
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
    #[token("!")]
    Not,

    // keywords
    #[token("let")]
    Let,

    // control flow
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,

    // fallback for invalid tokens
    #[regex(".", |lex| lex.slice().to_string(), priority = 0)]
    Invalid(String),
}
