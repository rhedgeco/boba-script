use std::collections::VecDeque;

use boba_script::{
    lexer::{LexResult, LexerError, LexerState, LineLexer, TextLines},
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

pub struct ShellTokens {
    tokens: VecDeque<(Result<Token, LexerError>, Span)>,
    state: Option<LexerState>,
    source: String,
    span: Span,
}

impl ShellTokens {
    pub fn new() -> Self {
        Self {
            tokens: VecDeque::new(),
            state: None,
            source: String::new(),
            span: Span::from(0..0),
        }
    }

    pub fn close_all_blocks(&mut self) {
        if let Some(state) = &mut self.state {
            state.close_all_blocks();
        }
    }

    pub fn is_ready(&self) -> bool {
        match &self.state {
            Some(state) => state.is_finished(),
            None => true,
        }
    }

    pub fn load(&mut self, text: impl AsRef<str>) -> bool {
        let text = text.as_ref();
        self.source.push_str(text);
        let lines = TextLines::new(text);

        let mut loaded = false;
        for line in lines {
            let mut lexer = match self.state.take() {
                Some(state) => LineLexer::new_with(line, state),
                None => LineLexer::new(line),
            };

            loop {
                match lexer.generate() {
                    LexResult::Finished => break,
                    LexResult::Token(token) => {
                        loaded = true;
                        let span = lexer.span();
                        self.tokens.push_back((Ok(token), span));
                    }
                    LexResult::Error(error) => {
                        loaded = true;
                        let span = lexer.span();
                        self.tokens.push_back((Err(error), span));
                    }
                    LexResult::Incomplete => {
                        self.state = Some(lexer.consume());
                        break;
                    }
                }
            }
        }

        loaded
    }
}

impl Iterator for ShellTokens {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        let (result, span) = self.tokens.pop_front()?;
        self.span = span;
        Some(result)
    }
}

impl TokenStream for ShellTokens {
    type Error = LexerError;
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
