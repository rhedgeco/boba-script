use derive_more::Display;
use once_cell::sync::Lazy;
use regex::Regex;

use crate::{
    parser::{report::PError, TokenSource},
    token::Span,
    Token,
};

use super::Spanned;

#[derive(Debug, Display, Clone)]
#[display(fmt = "{}", ident)]
pub struct Ident {
    ident: String,
    span: Span,
}

impl AsRef<str> for Ident {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Spanned for Ident {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Ident {
    pub fn parse(source: &mut TokenSource) -> Result<Self, PError> {
        match source.take() {
            Some((Token::Ident(str), span)) => Self::parse_str(str, span),
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

    pub fn parse_str(str: impl AsRef<str>, span: Span) -> Result<Self, PError> {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^[_a-zA-Z][_a-zA-z0-9]*$").expect("Invalid Ident regex"));

        let ident = str.as_ref().to_string();
        match REGEX.is_match(&ident) {
            false => Err(PError::InvalidIdent { ident, span }),
            true => Ok(Self { ident, span }),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.ident
    }
}
