use std::collections::VecDeque;

use boba_script::{
    lexer::{LexError, Lexer},
    parser::{stream::SourceSpan, token::Span, Token, TokenStream},
};

#[derive(Debug, Clone, Copy)]
pub struct ShellSource {
    span: Span,
}

impl SourceSpan for ShellSource {
    fn start(&self) -> usize {
        self.span.start
    }

    fn end(&self) -> usize {
        self.span.end
    }

    fn build(&self, span: impl Into<Span>) -> Self {
        Self { span: span.into() }
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
        ShellSource { span: span.into() }
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

    pub fn load(&mut self, text: impl AsRef<str>) {
        // start lexing the text
        let text = text.as_ref();
        let mut tokens = self.lexer.lex(text);

        // load all the tokens
        let mut loaded = false;
        while let Some(result) = tokens.next() {
            let span = tokens.token_span();
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
    }
}
