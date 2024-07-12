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

pub struct LineParser<Stream: TokenStream> {
    peeked: Option<Token>,
    stream: Stream,
    span: Span,
}

impl<Stream: TokenStream> LineParser<Stream> {
    pub fn new(stream: Stream) -> Self {
        Self {
            peeked: None,
            span: stream.token_end_span(),
            stream,
        }
    }

    pub fn line(&mut self) -> TokenLine<Stream> {
        TokenLine { parser: self }
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

    pub fn take_token(&mut self) -> Option<Result<Token, PError<Stream>>> {
        // try to take the peeked token first
        if let Some(token) = self.peeked.take() {
            self.span = self.stream.token_span();
            return Some(Ok(token));
        }

        // then generate the next one
        match self.generate() {
            None => {
                self.span = self.token_end_span();
                None
            }
            Some(result) => {
                self.span = self.stream.token_span();
                Some(result)
            }
        }
    }

    pub fn peek_token(&mut self) -> Option<Result<&Token, PError<Stream>>> {
        match &self.peeked {
            Some(_) => match &self.peeked {
                Some(token) => Some(Ok(token)),
                _ => unreachable!(),
            },
            None => match self.generate()? {
                Err(error) => Some(Err(error)),
                Ok(token) => Some(Ok(self.peeked.insert(token))),
            },
        }
    }

    pub fn consume_line(&mut self) -> Result<(), Vec<PError<Stream>>> {
        let mut errors = Vec::new();
        while let Some(result) = self.take_token() {
            match result {
                Err(error) => errors.push(error),
                Ok(Token::Newline) => break,
                Ok(_) => continue,
            }
        }

        match errors.is_empty() {
            false => Err(errors),
            true => Ok(()),
        }
    }
}

pub struct ErrorLine<'a, Stream: TokenStream> {
    errors: Vec<PError<Stream>>,
    line: TokenLine<'a, Stream>,
}

impl<'a, Stream: TokenStream> ErrorLine<'a, Stream> {
    pub fn token_start(&self) -> usize {
        self.line.token_start()
    }

    pub fn token_end(&self) -> usize {
        self.line.token_end()
    }

    pub fn token_span(&self) -> Span {
        self.line.token_span()
    }

    pub fn token_start_span(&self) -> Span {
        self.line.token_start_span()
    }

    pub fn token_end_span(&self) -> Span {
        self.line.token_end_span()
    }

    pub fn build_source(&self, span: impl Into<Span>) -> Stream::Source {
        self.line.build_source(span)
    }

    pub fn token_source(&self) -> Stream::Source {
        self.line.token_source()
    }

    pub fn token_start_source(&self) -> Stream::Source {
        self.line.token_start_source()
    }

    pub fn token_end_source(&self) -> Stream::Source {
        self.line.token_end_source()
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

pub struct LinePeeker<'a, Stream: TokenStream> {
    line: TokenLine<'a, Stream>,
}

impl<'a, Stream: TokenStream> LinePeeker<'a, Stream> {
    pub fn token(&self) -> Option<&Token> {
        match &self.line.parser.peeked {
            Some(Token::Newline) | None => None,
            Some(token) => Some(token),
        }
    }

    pub fn ignore(self) -> TokenLine<'a, Stream> {
        self.line
    }

    pub fn consume(mut self) -> TokenLine<'a, Stream> {
        self.line.take_token();
        self.line
    }

    pub fn take(self) -> (Option<Token>, TokenLine<'a, Stream>) {
        match self.line.parser.peeked.take() {
            None => (None, self.line),
            Some(Token::Newline) => {
                self.line.parser.peeked = Some(Token::Newline);
                (None, self.line)
            }
            Some(token) => (Some(token), self.line),
        }
    }
}

pub struct TokenLine<'a, Stream: TokenStream> {
    parser: &'a mut LineParser<Stream>,
}

impl<'a, Stream: TokenStream> TokenLine<'a, Stream> {
    pub fn token_start(&self) -> usize {
        self.parser.token_start()
    }

    pub fn token_end(&self) -> usize {
        self.parser.token_end()
    }

    pub fn token_span(&self) -> Span {
        self.parser.token_span()
    }

    pub fn token_start_span(&self) -> Span {
        self.parser.token_start_span()
    }

    pub fn token_end_span(&self) -> Span {
        self.parser.token_end_span()
    }

    pub fn build_source(&self, span: impl Into<Span>) -> Stream::Source {
        self.parser.build_source(span)
    }

    pub fn token_source(&self) -> Stream::Source {
        self.parser.token_source()
    }

    pub fn token_start_source(&self) -> Stream::Source {
        self.parser.token_start_source()
    }

    pub fn token_end_source(&self) -> Stream::Source {
        self.parser.token_end_source()
    }

    pub fn take_token(&mut self) -> Option<Result<Token, PError<Stream>>> {
        match self.parser.take_token()? {
            // if a newline is found, then the line is complete
            // store the newline back in the parser, and return none
            Ok(Token::Newline) => {
                self.parser.peeked = Some(Token::Newline);
                None
            }
            // otherwise return the result
            result => Some(result),
        }
    }

    pub fn peek_token(&mut self) -> Option<Result<&Token, PError<Stream>>> {
        match self.parser.peek_token()? {
            Ok(Token::Newline) => None,
            result => Some(result),
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

    pub fn peek_some(&mut self, expect: impl Into<String>) -> Result<&Token, PError<Stream>> {
        match self.peek_token() {
            Some(_) => match self.peek_token() {
                Some(result) => result,
                _ => unreachable!(),
            },
            None => Err(ParseError::UnexpectedInput {
                expect: expect.into(),
                found: None,
                source: self.parser.stream.token_source(),
            }),
        }
    }

    pub fn take_expect(&mut self, expect: Option<&Token>) -> Result<(), PError<Stream>> {
        let token = match self.take_token() {
            None => None,
            Some(Ok(token)) => Some(token),
            Some(Err(error)) => return Err(error),
        };

        match expect == token.as_ref() {
            true => Ok(()),
            false => Err(ParseError::UnexpectedInput {
                expect: match expect {
                    Some(token) => format!("{token}"),
                    None => format!("end of line"),
                },
                found: token,
                source: self.token_source(),
            }),
        }
    }

    pub fn peek_expect(&mut self, expect: Option<&Token>) -> Result<(), PError<Stream>> {
        let token = match self.peek_token() {
            None => None,
            Some(Ok(token)) => Some(token),
            Some(Err(error)) => return Err(error),
        };

        match expect == token {
            true => Ok(()),
            false => Err(ParseError::UnexpectedInput {
                expect: match expect {
                    Some(token) => format!("{token}"),
                    None => format!("end of line"),
                },
                found: token.cloned(),
                source: self.token_source(),
            }),
        }
    }

    pub fn take_guard<O>(
        &mut self,
        take: impl FnOnce(Option<Token>, &mut Self) -> Result<O, Vec<PError<Stream>>>,
    ) -> Result<O, Vec<PError<Stream>>> {
        self.take_guard_else(take, |_| {})
    }

    pub fn peek_guard<O>(
        &mut self,
        peek: impl FnOnce(LinePeeker<Stream>) -> Result<O, Vec<PError<Stream>>>,
    ) -> Result<O, Vec<PError<Stream>>> {
        self.peek_guard_else(peek, |_| {})
    }

    pub fn guard_else<O>(
        &mut self,
        guard: impl FnOnce(&mut Self) -> Result<O, Vec<PError<Stream>>>,
        on_error: impl FnOnce(&mut ErrorLine<Stream>),
    ) -> Result<O, Vec<PError<Stream>>> {
        match guard(self) {
            Ok(output) => Ok(output),
            Err(errors) => {
                let mut line = ErrorLine {
                    line: self.parser.line(),
                    errors,
                };
                on_error(&mut line);
                Err(line.errors)
            }
        }
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

    pub fn peek_guard_else<O>(
        &mut self,
        peek: impl FnOnce(LinePeeker<Stream>) -> Result<O, Vec<PError<Stream>>>,
        on_error: impl FnOnce(&mut ErrorLine<Stream>),
    ) -> Result<O, Vec<PError<Stream>>> {
        self.guard_else(
            |line| match line.peek_token() {
                Some(Err(error)) => Err(vec![error]),
                None | Some(Ok(_)) => peek(LinePeeker {
                    line: line.parser.line(),
                }),
            },
            on_error,
        )
    }

    pub fn consume_line(&mut self, store: &mut Vec<PError<Stream>>) {
        self.consume_until(store, |_| ConsumeFlag::Ignore);
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
                    ConsumeFlag::Exclusive => {
                        let token = self.parser.peeked.insert(token);
                        return ConsumeEnd::Exclusive(token);
                    }
                },
            }
        }

        ConsumeEnd::Nothing
    }
}
