use crate::{error::ParseError, token::Span, PError, Token};

pub type StreamSpan<T> = <<T as TokenStream>::Source as Source>::Span;

pub trait SourceSpan {
    fn span(&self) -> Span;
    fn start(&self) -> usize;
    fn end(&self) -> usize;
}

pub trait Source {
    type Span: SourceSpan;
    fn text(&self) -> &str;
    fn span(&self, span: impl Into<Span>) -> Self::Span;
}

pub trait TokenStream: Iterator<Item = Result<Token, Self::Error>> {
    type Error;
    type Source: Source;
    fn span(&self) -> Span;
    fn source(&self) -> &Self::Source;
    fn parser(self) -> TokenParser<Self>
    where
        Self: Sized,
    {
        TokenParser::new(self)
    }
}

pub struct TokenParser<T: TokenStream> {
    peeked: Option<(Token, Span)>,
    span: Span,
    stream: T,
}

impl<T: TokenStream> TokenParser<T> {
    pub fn new(stream: T) -> Self {
        Self {
            peeked: None,
            span: stream.span(),
            stream,
        }
    }

    pub fn stream(&self) -> &T {
        &self.stream
    }

    pub fn token_start(&self) -> usize {
        self.span.start
    }

    pub fn token_end(&self) -> usize {
        self.span.end
    }

    pub fn token_span(&self) -> StreamSpan<T> {
        self.stream.source().span(self.span)
    }

    pub fn token_end_span(&self) -> StreamSpan<T> {
        self.stream.source().span(self.span.end..self.span.end)
    }

    pub fn token_start_span(&self) -> StreamSpan<T> {
        self.stream.source().span(self.span.start..self.span.start)
    }

    pub fn span(&self, span: impl Into<Span>) -> StreamSpan<T> {
        self.stream.source().span(span.into())
    }

    pub fn take_next(&mut self) -> Option<Result<Token, PError<T>>> {
        if let Some((token, span)) = self.peeked.take() {
            self.span = span;
            return Some(Ok(token));
        };

        let result = self.stream.next()?;
        self.span = self.stream.span();
        match result {
            Ok(token) => Some(Ok(token)),
            Err(error) => {
                let span = self.span(self.stream.span());
                Some(Err(ParseError::TokenError { error, span }))
            }
        }
    }

    pub fn peek_next(&mut self) -> Option<Result<&Token, PError<T>>> {
        match &self.peeked {
            Some(_) => match &self.peeked {
                Some((token, _)) => Some(Ok(token)),
                _ => unreachable!(),
            },
            None => match self.stream.next()? {
                Ok(token) => {
                    let span = self.stream.span();
                    let (token, _) = self.peeked.insert((token, span));
                    Some(Ok(token))
                }
                Err(error) => {
                    let span = self.span(self.stream.span());
                    Some(Err(ParseError::TokenError { error, span }))
                }
            },
        }
    }

    pub fn take_expect(&mut self, expect: Option<&Token>) -> Result<(), PError<T>> {
        let token = match self.take_next() {
            None => None,
            Some(Ok(token)) => Some(token),
            Some(Err(error)) => return Err(error),
        };

        match expect == token.as_ref() {
            true => Ok(()),
            false => match expect {
                Some(expect) => Err(ParseError::UnexpectedInput {
                    expect: expect.to_string(),
                    found: token,
                    span: self.token_span(),
                }),
                None => Err(ParseError::UnexpectedInput {
                    expect: "end of input".into(),
                    found: token,
                    span: self.token_span(),
                }),
            },
        }
    }

    pub fn peek_expect(&mut self, expect: Option<&Token>) -> Result<(), PError<T>> {
        let token = match self.peek_next() {
            None => None,
            Some(Ok(token)) => Some(token),
            Some(Err(error)) => return Err(error),
        };

        match expect == token {
            true => Ok(()),
            false => match expect {
                Some(expect) => Err(ParseError::UnexpectedInput {
                    expect: expect.to_string(),
                    found: token.cloned(),
                    span: self.span(self.stream.span()),
                }),
                None => Err(ParseError::UnexpectedInput {
                    expect: "end of input".into(),
                    found: token.cloned(),
                    span: self.token_end_span(),
                }),
            },
        }
    }

    pub fn parse_next<O>(
        &mut self,
        validate: impl FnOnce(Option<Token>, &mut Self) -> Result<O, Vec<PError<T>>>,
    ) -> Result<O, Vec<PError<T>>> {
        self.parse_next_else(validate, |_| {})
    }

    pub fn parse_peek<O>(
        &mut self,
        validate: impl FnOnce(PeekParser<T>) -> Result<O, Vec<PError<T>>>,
    ) -> Result<O, Vec<PError<T>>> {
        self.parse_peek_else(validate, |_| {})
    }

    pub fn parse_else<O>(
        &mut self,
        parse: impl FnOnce(&mut TokenParser<T>) -> Result<O, Vec<PError<T>>>,
        on_error: impl FnOnce(&mut ErrorParser<T>),
    ) -> Result<O, Vec<PError<T>>> {
        match parse(self) {
            Ok(output) => Ok(output),
            Err(errors) => {
                let mut parser = ErrorParser {
                    parser: self,
                    errors,
                };
                on_error(&mut parser);
                Err(parser.errors)
            }
        }
    }

    pub fn parse_next_else<O>(
        &mut self,
        validate: impl FnOnce(Option<Token>, &mut Self) -> Result<O, Vec<PError<T>>>,
        on_error: impl FnOnce(&mut ErrorParser<T>),
    ) -> Result<O, Vec<PError<T>>> {
        self.parse_else(
            |parser| {
                let token = match parser.take_next() {
                    Some(Err(error)) => return Err(vec![error]),
                    Some(Ok(token)) => Some(token),
                    None => None,
                };

                validate(token, parser)
            },
            on_error,
        )
    }

    pub fn parse_peek_else<O>(
        &mut self,
        validate: impl FnOnce(PeekParser<T>) -> Result<O, Vec<PError<T>>>,
        on_error: impl FnOnce(&mut ErrorParser<T>),
    ) -> Result<O, Vec<PError<T>>> {
        self.parse_else(
            |parser| {
                match parser.peek_next() {
                    Some(Err(error)) => return Err(vec![error]),
                    Some(Ok(token)) => Some(token),
                    None => None,
                };

                validate(PeekParser { parser })
            },
            on_error,
        )
    }

    pub fn consume_until(
        &mut self,
        store: &mut Vec<PError<T>>,
        until: impl Fn(&Token) -> bool,
    ) -> Option<&Token> {
        while let Some(result) = self.peek_next() {
            match result {
                Err(error) => store.push(error),
                Ok(token) => match until(&token) {
                    true => match self.peek_next() {
                        Some(Ok(token)) => return Some(token),
                        _ => unreachable!(),
                    },
                    false => {
                        self.take_next();
                    }
                },
            }
        }

        None
    }
}

pub struct PeekParser<'a, T: TokenStream> {
    parser: &'a mut TokenParser<T>,
}

impl<'a, T: TokenStream> PeekParser<'a, T> {
    pub fn token(&self) -> Option<&Token> {
        let (token, _) = self.parser.peeked.as_ref()?;
        Some(token)
    }

    pub fn token_span(&self) -> StreamSpan<T> {
        match self.parser.peeked.as_ref() {
            Some((_, span)) => self.parser.span(*span),
            None => self.parser.token_end_span(),
        }
    }

    pub fn pair(&self) -> Option<(&Token, StreamSpan<T>)> {
        let (token, span) = self.parser.peeked.as_ref()?;
        Some((token, self.parser.span(*span)))
    }

    pub fn ignore(self) -> &'a mut TokenParser<T> {
        self.parser
    }

    pub fn consume(self) -> &'a mut TokenParser<T> {
        self.take().1
    }

    pub fn take(self) -> (Option<Token>, &'a mut TokenParser<T>) {
        match self.parser.peeked.take() {
            None => (None, self.parser),
            Some((token, span)) => {
                self.parser.span = span;
                (Some(token), self.parser)
            }
        }
    }
}

pub struct ErrorParser<'a, T: TokenStream> {
    errors: Vec<PError<T>>,
    parser: &'a mut TokenParser<T>,
}

impl<'a, T: TokenStream> ErrorParser<'a, T> {
    pub fn token_start(&self) -> usize {
        self.parser.token_start()
    }

    pub fn token_end(&self) -> usize {
        self.parser.token_end()
    }

    pub fn token_span(&self) -> StreamSpan<T> {
        self.parser.token_span()
    }

    pub fn token_end_span(&self) -> StreamSpan<T> {
        self.parser.token_end_span()
    }

    pub fn token_start_span(&self) -> StreamSpan<T> {
        self.parser.token_start_span()
    }

    pub fn span(&self, span: impl Into<Span>) -> StreamSpan<T> {
        self.parser.span(span)
    }

    pub fn push(&mut self, error: PError<T>) {
        self.errors.push(error)
    }

    pub fn consume_next(&mut self) {
        match self.parser.take_next() {
            Some(Err(error)) => self.errors.push(error),
            Some(Ok(_)) | None => (),
        }
    }

    pub fn consume_until(&mut self, until: impl Fn(&Token) -> bool) -> Option<&Token> {
        self.parser.consume_until(&mut self.errors, until)
    }
}
