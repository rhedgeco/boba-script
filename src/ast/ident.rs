use derive_more::Display;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    parser::{report::PError, Node, TokenSource},
    Token,
};

#[derive(Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[display(fmt = "{}", _0)]
pub struct Ident(String);
impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Ident {
    pub fn parse<'a>(source: &mut impl TokenSource<'a>) -> Result<Node<Self>, PError> {
        match source.take() {
            Some((Token::Ident(str), span)) => match Self::parse_str(str) {
                Some(ident) => Ok(Node::build(span, ident)),
                None => Err(PError::InvalidIdent {
                    ident: str.into(),
                    span,
                }),
            },
            Some((token, span)) => {
                return Err(PError::UnexpectedToken {
                    expect: "identifier".into(),
                    found: format!("'{token}'"),
                    span: span.clone(),
                })
            }
            None => {
                return Err(PError::UnexpectedEnd {
                    expect: "identifier".into(),
                    pos: source.pos(),
                })
            }
        }
    }

    pub fn parse_str(ident: impl AsRef<str>) -> Option<Self> {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^[_a-zA-Z][_a-zA-z0-9]*$").expect("Invalid Ident regex"));

        let ident = ident.as_ref();
        REGEX.is_match(ident).then(|| Self(ident.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
