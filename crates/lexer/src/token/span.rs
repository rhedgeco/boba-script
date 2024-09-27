use std::ops::Range;

use derive_more::derive::Display;

#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[display("{start}..{end}")]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Into<Range<usize>> for Span {
    fn into(self) -> Range<usize> {
        self.start..self.end
    }
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Span::new(value.start, value.end)
    }
}

impl From<Range<&usize>> for Span {
    fn from(value: Range<&usize>) -> Self {
        Span::new(*value.start, *value.end)
    }
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn range(&self) -> Range<usize> {
        self.start..self.end
    }

    pub fn range_ref(&self) -> Range<&usize> {
        &self.start..&self.end
    }
}
