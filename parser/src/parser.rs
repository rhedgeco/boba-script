use crate::{error::PError, stream::StreamExt, token::Span, ParseError, Token, TokenStream};

pub enum ConsumeFlag {
    Inclusive,
    Exclusive,
    Ignore,
}

pub enum ConsumeEnd<'a> {
    Inclusive(Token),
    Exclusive(&'a Token),
    Nothing,
}

pub struct ErrorLine<'a, 'source, Stream: TokenStream> {
    errors: Vec<PError<Stream>>,
    line: &'a mut TokenLine<'source, Stream>,
}

impl<'a, 'source, Stream: TokenStream> ErrorLine<'a, 'source, Stream> {
    pub fn line(&self) -> &TokenLine<'source, Stream> {
        &self.line
    }

    pub fn line_mut(&mut self) -> &mut TokenLine<'source, Stream> {
        &mut self.line
    }

    pub fn push(&mut self, error: PError<Stream>) {
        self.errors.push(error);
    }

    pub fn consume_line(&mut self) {
        self.line.consume_line(&mut self.errors);
    }

    pub fn consume_until(&mut self, until: impl Fn(&Token) -> ConsumeFlag) -> ConsumeEnd {
        self.line.consume_until(&mut self.errors, until)
    }
}

pub struct TokenLine<'a, Stream: TokenStream> {
    peeked: Option<Result<Token, PError<Stream>>>,
    stream: &'a mut Stream,
    span: Span,
}

impl<'a, Stream: TokenStream> TokenLine<'a, Stream> {
    pub fn new(stream: &'a mut Stream) -> Self {
        Self {
            peeked: None,
            span: stream.token_start_span(),
            stream,
        }
    }

    pub fn token_start(&self) -> usize {
        self.span.start
    }

    pub fn token_end(&self) -> usize {
        self.span.end
    }

    pub fn token_span(&self) -> Span {
        self.span
    }

    pub fn token_start_span(&self) -> Span {
        (self.token_start()..self.token_start()).into()
    }

    pub fn token_end_span(&self) -> Span {
        (self.token_end()..self.token_end()).into()
    }

    pub fn build_source(&self, span: impl Into<Span>) -> Stream::Source {
        self.stream.build_source(span)
    }

    pub fn token_source(&self) -> Stream::Source {
        self.build_source(self.token_span())
    }

    pub fn token_start_source(&self) -> Stream::Source {
        self.build_source(self.token_start_span())
    }

    pub fn token_end_source(&self) -> Stream::Source {
        self.build_source(self.token_end_span())
    }

    fn generate(&mut self) -> Option<Result<Token, PError<Stream>>> {
        match self.stream.next()? {
            Ok(token) => Some(Ok(token)),
            Err(error) => Some(Err(ParseError::TokenError {
                error,
                source: self.stream.token_source(),
            })),
        }
    }

    pub fn consume_token(&mut self) {
        self.take_token();
    }

    pub fn take_token(&mut self) -> Option<Result<Token, PError<Stream>>> {
        // take peeked token, or generate a new one
        let result = match self.peeked.take() {
            Some(result) => result,
            None => self.generate()?,
        };

        match result {
            // if the token is a newline,
            // store it and return none instead
            Ok(Token::Newline) => {
                self.peeked = Some(result);
                None
            }
            // otherwise just return the result
            _ => Some(result),
        }
    }

    pub fn peek_token(&mut self) -> Option<&Result<Token, PError<Stream>>> {
        // check the peeked result
        match &self.peeked {
            // if its a newline, just return none
            Some(Ok(Token::Newline)) => None,

            // if something else, just return it
            Some(_) => match &self.peeked {
                Some(result) => Some(result),
                _ => unreachable!(),
            },

            // if there is no peeked token, generate a new one
            None => {
                let result = self.generate()?;
                match self.peeked.insert(result) {
                    // still return none for newlines
                    Ok(Token::Newline) => None,
                    result => Some(result),
                }
            }
        }
    }

    pub fn take_some(&mut self, expect: impl Into<String>) -> Result<Token, PError<Stream>> {
        match self.take_token() {
            Some(result) => result,
            None => Err(ParseError::UnexpectedInput {
                expect: expect.into(),
                found: None,
                source: self.token_source(),
            }),
        }
    }

    pub fn take_exact(&mut self, exact: Option<&Token>) -> Result<(), PError<Stream>> {
        let token = match self.take_token() {
            Some(Ok(token)) => Some(token),
            Some(Err(error)) => return Err(error),
            None => None,
        };

        match token.as_ref() == exact {
            true => Ok(()),
            false => Err(ParseError::UnexpectedInput {
                expect: match &exact {
                    Some(token) => format!("{token}"),
                    None => format!("end of line"),
                },
                found: token,
                source: self.token_source(),
            }),
        }
    }

    pub fn consume_line(&mut self, store: &mut Vec<PError<Stream>>) {
        while let Some(result) = self.take_token() {
            if let Err(error) = result {
                store.push(error);
            }
        }
    }

    pub fn consume_until(
        &mut self,
        store: &mut Vec<PError<Stream>>,
        until: impl Fn(&Token) -> ConsumeFlag,
    ) -> ConsumeEnd {
        while let Some(result) = self.take_token() {
            match result {
                Err(error) => store.push(error),
                Ok(token) => match until(&token) {
                    ConsumeFlag::Ignore => continue,
                    ConsumeFlag::Inclusive => return ConsumeEnd::Inclusive(token),
                    ConsumeFlag::Exclusive => match self.peeked.insert(Ok(token)) {
                        Ok(token) => return ConsumeEnd::Exclusive(token),
                        _ => unreachable!(),
                    },
                },
            }
        }

        ConsumeEnd::Nothing
    }

    pub fn guard_else<O>(
        &mut self,
        guard: impl FnOnce(&mut Self) -> Result<O, Vec<PError<Stream>>>,
        on_error: impl FnOnce(&mut ErrorLine<Stream>),
    ) -> Result<O, Vec<PError<Stream>>> {
        match guard(self) {
            Ok(output) => Ok(output),
            Err(errors) => {
                let mut line = ErrorLine { line: self, errors };
                on_error(&mut line);
                Err(line.errors)
            }
        }
    }

    pub fn take_guard<O>(
        &mut self,
        take: impl FnOnce(Option<Token>, &mut Self) -> Result<O, Vec<PError<Stream>>>,
    ) -> Result<O, Vec<PError<Stream>>> {
        self.take_guard_else(take, |_| {})
    }

    pub fn take_guard_else<O>(
        &mut self,
        take: impl FnOnce(Option<Token>, &mut Self) -> Result<O, Vec<PError<Stream>>>,
        on_error: impl FnOnce(&mut ErrorLine<Stream>),
    ) -> Result<O, Vec<PError<Stream>>> {
        self.guard_else(
            |line| match line.take_token() {
                Some(Err(error)) => Err(vec![error]),
                Some(Ok(token)) => take(Some(token), line),
                None => take(None, line),
            },
            on_error,
        )
    }
}
