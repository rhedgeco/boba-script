use logos::{Logos, SpannedIter};

use crate::{token::Span, Token};

pub trait TokenSource<'source> {
    fn pos(&self) -> usize;
    fn source(&self) -> &str;
    fn peek(&mut self) -> Option<&(Token<'source>, Span)>;
    fn take(&mut self) -> Option<(Token<'source>, Span)>;
}

pub struct BufferSource<'source> {
    buffer: &'source str,
    iter: SpannedIter<'source, Token<'source>>,
    peek: Option<(Token<'source>, Span)>,
    pos: usize,
}

impl<'source> BufferSource<'source> {
    pub fn new(buffer: &'source str) -> Self {
        Self {
            buffer,
            iter: Token::lexer(buffer).spanned(),
            peek: None,
            pos: 0,
        }
    }
}

impl<'source> TokenSource<'source> for BufferSource<'source> {
    fn pos(&self) -> usize {
        self.pos
    }

    fn source(&self) -> &str {
        &self.buffer
    }

    fn peek(&mut self) -> Option<&(Token<'source>, Span)> {
        if self.peek.is_none() {
            self.peek = self.take();
        }

        self.peek.as_ref()
    }

    fn take(&mut self) -> Option<(Token<'source>, Span)> {
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
