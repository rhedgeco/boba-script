use derive_more::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq)]
pub enum IndentType {
    #[display(fmt = "space")]
    Space,
    #[display(fmt = "tab")]
    Tab,
}

#[derive(Debug, Default, Display, Clone, Copy, PartialEq)]
pub enum LexError {
    #[default]
    #[display(fmt = "invalid symbol")]
    InvalidSymbol,
    #[display(fmt = "indentation contains invalid {} characters", _0)]
    InvalidIndent(IndentType),
    #[display(fmt = "unclosed string")]
    UnclosedString,
}
