use boba_script_lexer::{LexFilter, Lexer, LexerError, Token};

pub struct SpannedLexer<'a> {
    lexer: LexFilter<'a>,
}

impl<'a> SpannedLexer<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer: lexer.filtered(),
        }
    }
}

impl<'a> Iterator for SpannedLexer<'a> {
    type Item = Result<(usize, Token<'a>, usize), LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.lexer.next()? {
            Err(error) => Err(error),
            Ok(token) => {
                let span = self.lexer.token_span();
                Ok((span.start, token, span.end))
            }
        })
    }
}
