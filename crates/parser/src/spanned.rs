use boba_script_lexer::{LexFilter, Lexer, LexerError, Token};

pub struct SpannedLexer<'a> {
    lexer: LexFilter<'a>,
    end: bool,
}

impl<'a> SpannedLexer<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer: lexer.filtered(),
            end: false,
        }
    }
}

impl<'a> Iterator for SpannedLexer<'a> {
    type Item = Result<(usize, Token<'a>, usize), LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.lexer.next() {
            Some(result) => match result {
                Err(error) => Some(Err(error)),
                Ok(token) => {
                    let span = self.lexer.token_span();
                    Some(Ok((span.start, token, span.end)))
                }
            },
            None => match self.end {
                true => None,
                false => {
                    self.end = true;
                    let span = self.lexer.token_span();
                    Some(Ok((span.start, Token::Newline, span.end)))
                }
            },
        }
    }
}
