use logos::{Logos, Span};
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug)]
pub struct TokenLine {
    indent: usize,
    source: String,
    tokens: Box<[(Token, Span)]>,
}

impl TokenLine {
    pub fn parse_lines<'a>(source: impl Into<&'a str>) -> TokenLineIter<'a> {
        TokenLineIter {
            current: 0,
            span_offset: 0,
            lines: source.into().split_inclusive("\n").collect(),
        }
    }

    pub fn indent(&self) -> usize {
        self.indent
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn tokens(&self) -> &[(Token, Span)] {
        &self.tokens
    }
}

#[derive(Debug, Clone)]
pub struct TokenError {
    pub span: Span,
    pub message: String,
    pub label: String,
}

pub struct TokenLineIter<'a> {
    current: usize,
    span_offset: usize,
    lines: Box<[&'a str]>,
}

impl Iterator for TokenLineIter<'_> {
    type Item = Result<TokenLine, TokenError>;

    fn next(&mut self) -> Option<Self::Item> {
        let line = *self.lines.get(self.current)?;
        let current_span_offset = self.span_offset;
        self.span_offset += line.chars().count();
        self.current += 1;

        let mut tokens = Vec::new();
        for (token, span) in Token::lexer(line).spanned() {
            let offset_span = span.start + current_span_offset..span.end + current_span_offset;
            match token {
                Ok(token) => tokens.push((token, offset_span)),
                Err(_) => {
                    return Some(Err(TokenError {
                        span: offset_span,
                        message: format!("Invalid token found while parsing"),
                        label: format!("Invalid token '{}'", &line[span]),
                    }))
                }
            }
        }

        if tokens.is_empty() {
            return self.next();
        }

        Some(Ok(TokenLine {
            indent: line.chars().take_while(|c| *c == ' ').count(),
            source: line
                .strip_suffix("\n")
                .unwrap_or(line)
                .strip_suffix("\r")
                .unwrap_or(line)
                .to_string(),
            tokens: tokens.into_boxed_slice(),
        }))
    }
}

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
pub struct Comment(String);
impl AsRef<str> for Comment {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Comment {
    pub fn new(comment: impl AsRef<str>) -> Option<Self> {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"#.*").expect("Invalid Comment regex"));

        let comment = comment.as_ref();
        REGEX
            .is_match(comment)
            .then(|| Self(comment[1..].trim().to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Logos, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[logos(skip r"[ \t\n\r\f]")] // skip whitespace
pub enum Token {
    #[regex(r"#.*", |lex| Comment::new(lex.slice()))]
    Comment(Comment),
    #[regex(r"[_a-zA-Z][_a-zA-z0-9]*", |lex| Ident::new(lex.slice()))]
    Ident(Ident),

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
    #[token("pass")]
    Pass,

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
