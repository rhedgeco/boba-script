use std::collections::VecDeque;

use boba_script::{
    lexer::{LexError, Lexer},
    parser::{stream::SourceSpan, token::Span, Token, TokenStream},
};
use boba_script_ariadne as ariadne;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShellId;

#[derive(Debug, Clone, Copy)]
pub struct ShellSource {
    id: ShellId,
    span: Span,
}

impl ariadne::Span for ShellSource {
    type SourceId = ShellId;

    fn source(&self) -> &Self::SourceId {
        &self.id
    }

    fn start(&self) -> usize {
        self.span.start
    }

    fn end(&self) -> usize {
        self.span.end
    }
}

impl SourceSpan for ShellSource {
    fn start(&self) -> usize {
        self.span.start
    }

    fn end(&self) -> usize {
        self.span.end
    }

    fn build(&self, span: impl Into<Span>) -> Self {
        Self {
            id: ShellId,
            span: span.into(),
        }
    }
}

pub struct ShellStream {
    tokens: VecDeque<(Result<Token, LexError>, Span)>,
    source: String,
    lexer: Lexer,
    span: Span,
}

impl Iterator for ShellStream {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        let (result, span) = self.tokens.pop_front()?;
        self.span = span;
        Some(result)
    }
}

impl TokenStream for ShellStream {
    type Error = LexError;
    type Source = ShellSource;

    fn token_start(&self) -> usize {
        self.span.start
    }

    fn token_end(&self) -> usize {
        self.span.end
    }

    fn build_source(&self, span: impl Into<Span>) -> Self::Source {
        ShellSource {
            id: ShellId,
            span: span.into(),
        }
    }
}

impl ShellStream {
    pub fn new() -> Self {
        Self {
            tokens: VecDeque::new(),
            source: String::new(),
            lexer: Lexer::new(),
            span: Span::from(0..0),
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    pub fn build_cache(&self) -> AriadneCache {
        AriadneCache {
            source: ariadne::Source::from(self.source.as_str()),
        }
    }

    pub fn load(&mut self, text: impl AsRef<str>) {
        // start lexing the text
        let text = text.as_ref();
        let mut tokens = self.lexer.lex(text);

        // load all the tokens
        let mut loaded = false;
        let span_offset = self.source.len() + 1;
        while let Some(result) = tokens.next() {
            let mut span = tokens.token_span();
            span.start += span_offset;
            span.end += span_offset;
            self.tokens.push_back((result, span));
            loaded = true;
        }

        // if there were no tokens
        // reset the indent and try loading the dedent tokens
        if !loaded {
            for _ in 0..self.lexer.close_blocks() {
                let end = self.span.end;
                let span = Span::from(end..end);
                self.tokens.push_back((Ok(Token::Dedent), span))
            }
        }

        // load the text into the source
        self.source.push_str(&format!("\n{text}"));
    }
}

pub struct AriadneCache<'a> {
    source: ariadne::Source<&'a str>,
}

impl<'a> ariadne::Cache<ShellId> for AriadneCache<'a> {
    type Storage = &'a str;

    fn fetch(
        &mut self,
        _: &ShellId,
    ) -> Result<&boba_script_ariadne::Source<Self::Storage>, Box<dyn std::fmt::Debug + '_>> {
        Ok(&self.source)
    }

    fn display<'b>(&self, _: &'b ShellId) -> Option<Box<dyn std::fmt::Display + 'b>> {
        Some(Box::new("shell"))
    }
}
