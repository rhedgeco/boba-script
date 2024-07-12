use std::str::SplitInclusive;

use derive_more::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[display(fmt = "{}", _0)]
pub struct TextLine<'a>(&'a str);

impl AsRef<str> for TextLine<'_> {
    fn as_ref(&self) -> &str {
        self.0
    }
}

impl<'a> TextLine<'a> {
    pub fn new(text: &'a str) -> Self {
        match text.split_once('\n') {
            Some((text, _)) => Self(text),
            None => Self(text),
        }
    }

    pub fn text(&self) -> &'a str {
        self.0
    }
}

pub struct TextLines<'a> {
    lines: SplitInclusive<'a, char>,
}

impl<'a> Iterator for TextLines<'a> {
    type Item = TextLine<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.next()?;
        Some(TextLine(line))
    }
}

impl<'a> TextLines<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            lines: text.split_inclusive('\n'),
        }
    }
}
