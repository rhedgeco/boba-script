use derive_more::derive::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LexerError {
    #[display("invalid symbol")]
    InvalidSymbol,
    #[display("mixed tabs and spaces")]
    MixedIndent,
    #[display("unclosed string")]
    UnclosedString,
}
