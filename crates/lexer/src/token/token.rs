use derive_more::derive::Display;

use super::build::{FloatBuilder, IntBuilder, StrFormat};

#[derive(Debug, Display, Clone, Copy)]
pub enum Token<'a> {
    // WHITESPACE
    #[display("comment({_0})")]
    Comment(&'a str),
    #[display("newline")]
    Newline,
    #[display("indent")]
    Indent,
    #[display("dedent")]
    Dedent,

    // VALUES
    #[display("none")]
    None,
    #[display("bool({_0})")]
    Bool(bool),
    #[display("int({_0})")]
    Int(IntBuilder<'a>),
    #[display("float({_0})")]
    Float(FloatBuilder<'a>),
    #[display("str({_0})")]
    Str(StrFormat<'a>),
    #[display("ident({_0})")]
    Ident(&'a str),

    // KEYWORDS
    #[display("if")]
    If,
    #[display("while")]
    While,

    // BRACES
    #[display("(")]
    OpenParen,
    #[display(")")]
    CloseParen,
    #[display("[")]
    OpenSquare,
    #[display("]")]
    CloseSquare,

    // CONTROL
    #[display(".")]
    Dot,
    #[display("=")]
    Assign,
    #[display("?")]
    Question,
    #[display(":")]
    Colon,
    #[display("::")]
    DoubleColon,

    // OPERATORS
    #[display("+")]
    Add,
    #[display("-")]
    Sub,
    #[display("!")]
    Not,
    #[display("*")]
    Mul,
    #[display("/")]
    Div,
    #[display("%")]
    Mod,
    #[display("**")]
    Pow,
    #[display("==")]
    Eq,
    #[display("<")]
    Lt,
    #[display(">")]
    Gt,
    #[display("!=")]
    NEq,
    #[display("<=")]
    LtEq,
    #[display(">=")]
    GtEq,
    #[display("&&")]
    And,
    #[display("||")]
    Or,
}
