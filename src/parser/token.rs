use derive_more::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Token<'source> {
    #[display(fmt = "newline")]
    Newline,

    #[display(fmt = "{}", _0)]
    Ident(&'source str),

    // values
    #[display(fmt = "none")]
    None,
    #[display(fmt = "{}", _0)]
    Bool(bool),
    #[display(fmt = "int")]
    UInt(&'source str),
    #[display(fmt = "float")]
    UFloat(&'source str),
    #[display(fmt = "string")]
    String(&'source str),

    // operators
    #[display(fmt = "+")]
    Add,
    #[display(fmt = "-")]
    Sub,
    #[display(fmt = "*")]
    Mul,
    #[display(fmt = "/")]
    Div,
    #[display(fmt = "%")]
    Mod,
    #[display(fmt = "**")]
    Pow,
    #[display(fmt = ":=")]
    Walrus,

    // boolean operators
    #[display(fmt = "and")]
    And,
    #[display(fmt = "or")]
    Or,
    #[display(fmt = "!")]
    Not,
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

    // control tokens
    #[display(fmt = ":")]
    Colon,
    #[display(fmt = "?")]
    Question,
    #[display(fmt = ".")]
    Dot,
    #[display(fmt = ",")]
    Comma,
    #[display(fmt = "(")]
    OpenParen,
    #[display(fmt = ")")]
    CloseParen,
    #[display(fmt = "=")]
    Assign,
    #[display(fmt = "->")]
    Arrow,
    #[display(fmt = "let")]
    Let,
    #[display(fmt = "fn")]
    Fn,
    #[display(fmt = "while")]
    While,
}

impl<'source> Token<'source> {
    pub fn get_keyword(str: impl AsRef<str>) -> Option<Self> {
        static KEYWORDS: phf::Map<&str, Token> = phf::phf_map! {
            "let" => Token::Let,
            "fn" => Token::Fn,
            "true" => Token::Bool(true),
            "false" => Token::Bool(false),
            "and" => Token::And,
            "or" => Token::Or,
            "none" => Token::None,
            "while" => Token::While,
        };

        KEYWORDS.get(str.as_ref()).cloned()
    }
}
