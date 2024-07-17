use crate::{token::Span, Token};

pub trait SourceSpan: Clone {
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn build(&self, span: impl Into<Span>) -> Self;
}

impl<T: SourceSpan> SourceExt for T {}
pub trait SourceExt: SourceSpan {
    fn span(&self) -> Span {
        (self.start()..self.end()).into()
    }

    fn start_span(&self) -> Span {
        (self.start()..self.start()).into()
    }

    fn end_span(&self) -> Span {
        (self.end()..self.end()).into()
    }

    fn start_source(&self) -> Self {
        self.build(self.start_span())
    }

    fn end_source(&self) -> Self {
        self.build(self.end_span())
    }
}

pub trait TokenStream: Iterator<Item = Result<Token, Self::Error>> {
    type Error;
    type Source: SourceSpan;
    fn token_start(&self) -> usize;
    fn token_end(&self) -> usize;
    fn build_source(&self, span: impl Into<Span>) -> Self::Source;
}

impl<T: TokenStream> StreamExt for T {}
pub trait StreamExt: TokenStream {
    fn token_span(&self) -> Span {
        (self.token_start()..self.token_end()).into()
    }

    fn token_start_span(&self) -> Span {
        (self.token_start()..self.token_start()).into()
    }

    fn token_end_span(&self) -> Span {
        (self.token_end()..self.token_end()).into()
    }

    fn token_source(&self) -> Self::Source {
        self.build_source(self.token_span())
    }

    fn token_start_source(&self) -> Self::Source {
        self.build_source(self.token_start_span())
    }

    fn token_end_source(&self) -> Self::Source {
        self.build_source(self.token_end_span())
    }
}
