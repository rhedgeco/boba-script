use derive_more::Display;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[display(fmt = "{}", _0)]
pub struct Ident<'a>(&'a str);
impl<'a> AsRef<str> for Ident<'a> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> Ident<'a> {
    pub fn new(ident: &'a str) -> Option<Self> {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"^[_a-zA-Z][_a-zA-z0-9]*$").expect("Invalid Ident regex"));

        REGEX.is_match(ident).then(|| Self(ident))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
