use derive_more::Display;
use logos::Logos;

use super::Ident;

pub type Span = logos::Span;

fn parse_string(str: impl AsRef<str>) -> String {
    let str = str.as_ref();

    // remove quotes
    let str = match str.strip_prefix("'") {
        Some(stripped) => stripped.strip_suffix("'").unwrap_or(str),
        None => match str.strip_prefix("\"") {
            Some(stripped) => stripped.strip_suffix("\"").unwrap_or(str),
            None => str,
        },
    }
    .to_string();

    // TODO: parse escaped characters

    str
}

fn parse_newline(str: impl AsRef<str>) -> usize {
    str.as_ref()[1..].len()
}

#[derive(Logos, Debug, Display, Clone, PartialEq, PartialOrd)]
#[logos(skip r" ")] // skip whitespace
pub enum Token {
    // newlines
    #[regex(r"\n[ ]*", |lex| parse_newline(lex.slice()))]
    Newline(usize),

    // identifiers
    #[regex(r"[_a-zA-Z][_a-zA-z0-9]*", |lex| Ident::new(lex.slice()))]
    #[display(fmt = "identifier")]
    Ident(Ident),

    // values
    #[regex(r"true|false", |lex| lex.slice() == "true")]
    #[display(fmt = "bool")]
    Bool(bool),
    #[regex(r"[0-9]+", |lex| lex.slice().to_string())]
    #[display(fmt = "int")]
    Int(String),
    #[regex(r"[0-9]*\.[0-9]+", |lex| lex.slice().to_string())]
    #[display(fmt = "float")]
    Float(String),
    #[regex("('[^']*')|(\"[^\"]*\")", |lex| parse_string(lex.slice()))]
    #[display(fmt = "string")]
    String(String),

    // operators
    #[token("=")]
    #[display(fmt = "=")]
    Equal,
    #[token("+")]
    #[display(fmt = "+")]
    Add,
    #[token("-")]
    #[display(fmt = "-")]
    Sub,
    #[token("*")]
    #[display(fmt = "*")]
    Mul,
    #[token("/")]
    #[display(fmt = "/")]
    Div,
    #[token("**")]
    #[display(fmt = "**")]
    Pow,
    #[token("!")]
    #[display(fmt = "!")]
    Not,

    // keywords
    #[token("let")]
    #[display(fmt = "let")]
    Let,

    // control flow
    #[token("(")]
    #[display(fmt = "(")]
    OpenParen,
    #[token(")")]
    #[display(fmt = ")")]
    CloseParen,

    // fallback for invalid tokens
    #[regex(".", |lex| lex.slice().to_string(), priority = 0)]
    #[display(fmt = "{}", _0)]
    Invalid(String),
}
