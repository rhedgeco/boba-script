use crate::{token::Span, Lexer, LexerError, Token};

pub struct LexFilter<'source> {
    lexer: Lexer<'source>,
    newline: bool,
}

impl<'source> LexFilter<'source> {
    pub fn new(lexer: Lexer<'source>) -> Self {
        Self {
            lexer,
            newline: true,
        }
    }

    pub fn token_start(&self) -> usize {
        self.lexer.token_start()
    }

    pub fn token_end(&self) -> usize {
        self.lexer.token_end()
    }

    pub fn token_span(&self) -> Span {
        self.lexer.token_span()
    }

    pub fn token_start_span(&self) -> Span {
        self.lexer.token_start_span()
    }

    pub fn token_end_span(&self) -> Span {
        self.lexer.token_end_span()
    }
}

impl<'source> Iterator for LexFilter<'source> {
    type Item = Result<Token<'source>, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            return Some(match self.lexer.next()? {
                // if a comment is found, just skip it
                Ok(Token::Comment(_)) => continue,

                // skip any newlines that come after previous newlines
                Ok(Token::Newline) => match self.newline {
                    true => continue,
                    false => {
                        self.newline = true;
                        Ok(Token::Newline)
                    }
                },

                // otherwise reset the newline count and return the token
                result => {
                    self.newline = false;
                    result
                }
            });
        }
    }
}
