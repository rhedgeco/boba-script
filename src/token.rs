use derive_more::Display;
use logos::Logos;

pub type Span = logos::Span;

fn parse_string(str: &str) -> &str {
    // remove quotes
    let str = match str.strip_prefix("'") {
        Some(stripped) => stripped.strip_suffix("'").unwrap_or(str),
        None => match str.strip_prefix("\"") {
            Some(stripped) => stripped.strip_suffix("\"").unwrap_or(str),
            None => str,
        },
    };

    // TODO: parse escaped characters

    str
}

fn parse_newline(str: impl AsRef<str>) -> usize {
    str.as_ref().replace("\n", "").replace("\r", "").len()
}

#[derive(Logos, Debug, Display, Clone, PartialEq, PartialOrd)]
#[logos(skip r" ")] // skip whitespace
pub enum Token<'a> {
    // newlines
    #[regex(r"(\n|\r)[ ]*", |lex| parse_newline(lex.slice()))]
    #[display(fmt = "newline {}", _0)]
    Newline(usize),

    // identifiers
    #[regex(r"[_a-zA-Z][_a-zA-z0-9]*", |lex| lex.slice())]
    #[display(fmt = "identifier")]
    Ident(&'a str),

    // values
    #[regex(r"(unit|\(\))")]
    #[display(fmt = "unit")]
    Unit,
    #[regex(r"[0-9]+", |lex| lex.slice())]
    #[display(fmt = "int")]
    Int(&'a str),
    #[regex(r"[0-9]*\.[0-9]+", |lex| lex.slice())]
    #[display(fmt = "float")]
    Float(&'a str),
    #[regex(r"true|false", |lex| lex.slice() == "true")]
    #[display(fmt = "bool")]
    Bool(bool),
    #[regex("('[^']*')|(\"[^\"]*\")", |lex| parse_string(lex.slice()))]
    #[display(fmt = "string")]
    String(&'a str),

    // math operators
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
    #[token("%")]
    #[display(fmt = "%")]
    Mod,
    #[token("**")]
    #[display(fmt = "**")]
    Pow,

    // boolean operators
    #[token("!")]
    #[display(fmt = "!")]
    Bang,
    #[token("==")]
    #[display(fmt = "==")]
    Eq,
    #[token("<")]
    #[display(fmt = "<")]
    Lt,
    #[token(">")]
    #[display(fmt = ">")]
    Gt,
    #[token("!=")]
    #[display(fmt = "!=")]
    NEq,
    #[token("<=")]
    #[display(fmt = "<=")]
    LtEq,
    #[token(">=")]
    #[display(fmt = ">=")]
    GtEq,
    #[regex("(and|&&)")]
    #[display(fmt = "and")]
    And,
    #[regex(r"(or|\|\|)")]
    #[display(fmt = "or")]
    Or,

    // other operators
    #[token("=")]
    #[display(fmt = "=")]
    Assign,
    #[token(":=")]
    #[display(fmt = ":=")]
    Walrus,

    // keywords
    #[token("let")]
    #[display(fmt = "let")]
    Let,
    #[token("fn")]
    #[display(fmt = "fn")]
    Fn,
    #[token("if")]
    #[display(fmt = "if")]
    If,
    #[token("else")]
    #[display(fmt = "else")]
    Else,

    // control flow
    #[token("(")]
    #[display(fmt = "(")]
    OpenParen,
    #[token(")")]
    #[display(fmt = ")")]
    CloseParen,
    #[token(":")]
    #[display(fmt = ":")]
    Colon,

    #[regex(r".", |lex| lex.slice(), priority = 0)]
    #[display(fmt = "{}", _0)]
    Invalid(&'a str),
}
