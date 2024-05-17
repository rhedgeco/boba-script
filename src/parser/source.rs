use logos::{Logos, SpannedIter};

use crate::{token::Span, Token};

pub struct TokenSource<'source> {
    buffer: &'source str,
    iter: SpannedIter<'source, Token<'source>>,
    peek: Option<(Token<'source>, Span)>,
    pos: usize,
}

impl<'source> TokenSource<'source> {
    pub fn new(buffer: &'source str) -> Self {
        Self {
            buffer,
            iter: Token::lexer(buffer).spanned(),
            peek: None,
            pos: 0,
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn source(&self) -> &str {
        &self.buffer
    }

    pub fn peek(&mut self) -> Option<&(Token<'source>, Span)> {
        if self.peek.is_none() {
            self.peek = self.take();
        }

        self.peek.as_ref()
    }

    pub fn take(&mut self) -> Option<(Token<'source>, Span)> {
        match self.peek.take() {
            Some(next) => Some(next),
            None => {
                let (token, span) = self.iter.next()?;
                let token = token.expect("unhandled invalid token");
                self.pos = span.end;
                Some((token, span))
            }
        }
    }
}
