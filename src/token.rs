use logos::Logos;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Int(String);
impl AsRef<str> for Int {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Int {
    pub fn new(int: impl Into<String>) -> Option<Self> {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^-?[0-9]+$").expect("Invalid Int regex"));

        let int = int.into();
        REGEX.is_match(&int).then(|| Self(int))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StringConst(String);
impl AsRef<str> for StringConst {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl StringConst {
    pub fn new(string: impl AsRef<str>) -> Option<Self> {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("^\"[^\"]*\"$").expect("Invalid StringConst regex"));

        let string = string.as_ref();
        REGEX.is_match(&string).then(|| {
            let string = &string[1..string.len() - 1];
            Self(string.to_string())
        })
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SingleComment(String);
impl AsRef<str> for SingleComment {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl SingleComment {
    pub fn new(comment: impl AsRef<str>) -> Option<Self> {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^//.*$").expect("Invalid SingleComment regex"));

        let comment = comment.as_ref();
        REGEX.is_match(&comment).then(|| {
            let value = comment[2..].trim();
            Self(value.to_string())
        })
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MultiComment(Box<[String]>);

impl MultiComment {
    pub fn new(comment: impl AsRef<str>) -> Option<Self> {
        static REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r"^/\*[^*]*\*+(?:[^/*][^*]*\*+)*/$").expect("Invalid MultiComment regex")
        });

        let comment = comment.as_ref();
        REGEX.is_match(&comment).then(|| {
            let lines = comment[2..comment.len() - 2]
                .split("\r\n") // split at windows style line feeds
                .flat_map(|line| line.split("\n")) // split normal line feeds
                .map(|s| s.trim().to_string())
                .collect::<Box<[_]>>();
            Self(lines)
        })
    }

    pub fn lines(&self) -> &[String] {
        &self.0
    }
}

#[derive(Logos, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[logos(skip r"[ \t\n\r\f]")] // skip whitespace
pub enum Token {
    // comments
    #[regex("//.*", |lex| SingleComment::new(lex.slice()))]
    SingleComment(SingleComment),
    #[regex("/\\*[^*]*\\*+(?:[^/*][^*]*\\*+)*/", |lex| MultiComment::new(lex.slice()))]
    MultiComment(MultiComment),

    // identifiers
    #[regex("[_a-zA-Z][_a-zA-z0-9]*", |lex| Ident::new(lex.slice()))]
    Ident(Ident),

    // constants
    #[regex("\"[^\"]*\"", |lex| StringConst::new(lex.slice()))]
    String(StringConst),
    #[regex("-?[0-9]+", |lex| Int::new(lex.slice()))]
    Int(Int),

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

    // keywords
    #[token("fn")]
    Fn,
    #[token("mod")]
    Mod,

    // control flow
    #[token("::")]
    DoubleColon,
    #[token(":")]
    Colon,
    #[token(";")]
    SemiColon,
    #[token(",")]
    Comma,
    #[token(".")]
    Period,
    #[token("->")]
    Arrow,
    #[token("(")]
    OpenParen,
    #[token(")")]
    CloseParen,
    #[token("{")]
    OpenCurly,
    #[token("}")]
    CloseCurly,
    #[token("[")]
    OpenSquare,
    #[token("]")]
    CloseSquare,
    #[token("<")]
    OpenAngle,
    #[token(">")]
    CloseAngle,
}
