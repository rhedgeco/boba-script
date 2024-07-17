use std::ops::Range;

use boba_script_core::dashu::integer::IBig;
use derive_more::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[display(fmt = "{}..{}", start, end)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Into<Range<usize>> for Span {
    fn into(self) -> Range<usize> {
        self.start..self.end
    }
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Self::new(value.start, value.end)
    }
}

impl From<Range<&usize>> for Span {
    fn from(value: Range<&usize>) -> Self {
        Self::new(*value.start, *value.end)
    }
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

#[derive(Debug, Display, Clone, PartialEq, PartialOrd)]
pub enum Token {
    // BLOCKS
    #[display(fmt = "newline")]
    Newline,
    #[display(fmt = "indent")]
    Indent,
    #[display(fmt = "dedent")]
    Dedent,

    // IDENTIFIERS
    #[display(fmt = "{}", _0)]
    Ident(String),

    // VALUES
    #[display(fmt = "none")]
    None,
    #[display(fmt = "{}", _0)]
    Bool(bool),
    #[display(fmt = "{}", _0)]
    Int(IBig),
    #[display(fmt = "{}", _0)]
    Float(f64),
    #[display(fmt = "'{}'", _0)]
    String(String),

    // OPERATORS
    #[display(fmt = "+")]
    Add,
    #[display(fmt = "-")]
    Sub,
    #[display(fmt = "not")]
    Not,
    #[display(fmt = "*")]
    Mul,
    #[display(fmt = "/")]
    Div,
    #[display(fmt = "%")]
    Modulo,
    #[display(fmt = "**")]
    Pow,
    #[display(fmt = "==")]
    Eq,
    #[display(fmt = "<")]
    Lt,
    #[display(fmt = ">")]
    Gt,
    #[display(fmt = "!=")]
    NEq,
    #[display(fmt = "<=")]
    LtEq,
    #[display(fmt = ">=")]
    GtEq,
    #[display(fmt = "and")]
    And,
    #[display(fmt = "or")]
    Or,
    #[display(fmt = ":=")]
    Walrus,

    // CONTROL
    #[display(fmt = ".")]
    Period,
    #[display(fmt = ",")]
    Comma,
    #[display(fmt = "=")]
    Assign,
    #[display(fmt = ":")]
    Colon,
    #[display(fmt = ";")]
    SemiColon,
    #[display(fmt = "?")]
    Question,
    #[display(fmt = "(")]
    OpenParen,
    #[display(fmt = ")")]
    CloseParen,
    #[display(fmt = "{{")]
    OpenCurly,
    #[display(fmt = "}}")]
    CloseCurly,
    #[display(fmt = "[")]
    OpenSquare,
    #[display(fmt = "]")]
    CloseSquare,
    #[display(fmt = "->")]
    Arrow,
    #[display(fmt = "=>")]
    FatArrow,

    // KEYWORDS
    #[display(fmt = "let")]
    Let,
    #[display(fmt = "fn")]
    Fn,
    #[display(fmt = "if")]
    If,
    #[display(fmt = "else")]
    Else,
    #[display(fmt = "while")]
    While,
    #[display(fmt = "static")]
    Static,
    #[display(fmt = "const")]
    Const,
}

impl Token {
    pub fn parse_ident(str: impl AsRef<str>) -> Self {
        const KEYWORDS: phf::Map<&'static str, Token> = phf::phf_map! {
            "none" => Token::None,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "not" => Token::Not,
            "and" => Token::And,
            "or" => Token::Or,
            "let" => Token::Let,
            "fn" => Token::Fn,
            "if" => Token::If,
            "else" => Token::Else,
            "while" => Token::While,
            "static" => Token::Static,
            "const" => Token::Const,
        };

        match KEYWORDS.get(str.as_ref()) {
            None => Token::Ident(str.as_ref().to_string()),
            Some(token) => token.clone(),
        }
    }
}
