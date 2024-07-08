use crate::{error::SpanParseError, token::Span, ParseError, Token};

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
    fn stream_parser(self) -> StreamParser<Self>
    where
        Self: Sized,
    {
        StreamParser::new(self)
    }
}

pub struct StreamParser<T: TokenStream> {
    peeked: Option<(Token, Span)>,
    span: Span,
    stream: T,
}

impl<T: TokenStream> Iterator for StreamParser<T> {
    type Item = Result<Token, ParseError<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some((token, span)) = self.peeked.take() {
            self.span = span;
            return Some(Ok(token));
        };

        let (result, span) = self.generate()?;
        self.span = span;
        Some(result)
    }
}

impl<T: TokenStream> StreamParser<T> {
    pub fn new(stream: T) -> Self {
        Self {
            peeked: None,
            span: stream.span(),
            stream,
        }
    }

    pub fn source(&self) -> &T::Source {
        self.stream.source()
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
        self.source().span(self.span)
    }

    pub fn token_span_start(&self) -> StreamSpan<T> {
        let start = self.token_start();
        self.source().span(start..start)
    }

    pub fn token_span_end(&self) -> StreamSpan<T> {
        let end = self.token_end();
        self.source().span(end..end)
    }

    pub fn peek(&mut self) -> Option<Result<&Token, ParseError<T>>> {
        Some(self.peek_spanned()?.map(|(token, _)| token))
    }

    pub fn peek_spanned(&mut self) -> Option<Result<(&Token, StreamSpan<T>), ParseError<T>>> {
        if self.peeked.is_some() {
            let (token, span) = self.peeked.as_ref().unwrap();
            let span = self.source().span(span.clone());
            return Some(Ok((token, span)));
        }

        let (result, span) = self.generate()?;
        match result {
            Ok(token) => {
                let source_span = self.source().span(span.clone());
                let (token, _) = self.peeked.insert((token, span));
                Some(Ok((token, source_span)))
            }
            Err(error) => {
                self.span = span;
                Some(Err(error))
            }
        }
    }

    pub fn next_some(&mut self, expect: impl Into<String>) -> Result<Token, ParseError<T>> {
        match self.next() {
            Some(result) => result,
            None => Err(SpanParseError::UnexpectedInput {
                expect: expect.into(),
                found: None,
                span: self.token_span_end(),
            }),
        }
    }

    pub fn peek_some(&mut self, expect: impl Into<String>) -> Result<&Token, ParseError<T>> {
        self.peek_some_spanned(expect).map(|(token, _)| token)
    }

    pub fn peek_some_spanned(
        &mut self,
        expect: impl Into<String>,
    ) -> Result<(&Token, StreamSpan<T>), ParseError<T>> {
        let end = self.token_span_end();
        match self.peek_spanned() {
            Some(result) => result,
            None => Err(SpanParseError::UnexpectedInput {
                expect: expect.into(),
                found: None,
                span: end,
            }),
        }
    }

    pub fn next_expect(&mut self, token: Option<&Token>) -> Result<(), ParseError<T>> {
        match self.next() {
            Some(Err(error)) => Err(error),
            Some(Ok(found)) => match Some(&found) == token {
                true => Ok(()),
                false => Err(SpanParseError::UnexpectedInput {
                    expect: match token {
                        Some(token) => format!("{token}"),
                        None => format!("end of input"),
                    },
                    found: Some(found),
                    span: self.token_span(),
                }),
            },
            None => match token {
                None => Ok(()),
                Some(token) => Err(SpanParseError::UnexpectedInput {
                    expect: format!("{token}"),
                    found: None,
                    span: self.token_span_end(),
                }),
            },
        }
    }

    pub fn peek_expect(&mut self, token: Option<&Token>) -> Result<(), ParseError<T>> {
        match self.peek_spanned() {
            Some(Err(error)) => Err(error),
            Some(Ok((found, span))) => match Some(found) == token {
                true => Ok(()),
                false => Err(SpanParseError::UnexpectedInput {
                    expect: match token {
                        Some(token) => format!("{token}"),
                        None => format!("end of input"),
                    },
                    found: Some(found.clone()),
                    span,
                }),
            },
            None => match token {
                None => Ok(()),
                Some(token) => Err(SpanParseError::UnexpectedInput {
                    expect: format!("{token}"),
                    found: None,
                    span: self.token_span_end(),
                }),
            },
        }
    }

    pub fn consume_until(
        &mut self,
        until: impl Fn(&Token) -> bool,
    ) -> Result<(), Vec<ParseError<T>>> {
        let mut errors = Vec::new();
        self.consume_until_with(&mut errors, until);
        match errors.is_empty() {
            false => Err(errors),
            true => Ok(()),
        }
    }

    pub fn consume_until_with(
        &mut self,
        errors: &mut Vec<ParseError<T>>,
        until: impl Fn(&Token) -> bool,
    ) {
        while let Some(result) = self.peek() {
            match result {
                Err(error) => errors.push(error),
                Ok(token) => match until(token) {
                    true => break,
                    false => {
                        self.next();
                    }
                },
            }
        }
    }

    pub fn consume_line(&mut self) -> Result<(), Vec<ParseError<T>>> {
        self.consume_until(|t| t == &Token::Newline)
    }

    pub fn consume_line_with(&mut self, errors: &mut Vec<ParseError<T>>) {
        self.consume_until_with(errors, |t| t == &Token::Newline);
    }

    pub fn next_line_end(&mut self) -> Result<(), ParseError<T>> {
        match self.next() {
            Some(Err(error)) => Err(error),
            Some(Ok(Token::Newline)) | None => Ok(()),
            Some(Ok(found)) => Err(SpanParseError::UnexpectedInput {
                expect: "line end".into(),
                found: Some(found),
                span: self.token_span(),
            }),
        }
    }

    pub fn peek_line_end(&mut self) -> Result<(), ParseError<T>> {
        match self.peek() {
            Some(Err(error)) => Err(error),
            Some(Ok(Token::Newline)) | None => Ok(()),
            Some(Ok(found)) => Err(SpanParseError::UnexpectedInput {
                expect: "line end".into(),
                found: Some(found.clone()),
                span: self.token_span(),
            }),
        }
    }

    // keep private, only used for internal token generation
    fn generate(&mut self) -> Option<(Result<Token, ParseError<T>>, Span)> {
        match self.stream.next()? {
            Ok(token) => Some((Ok(token), self.stream.span())),
            Err(error) => {
                let span = self.stream.span();
                Some((
                    Err(SpanParseError::TokenError {
                        error,
                        span: self.token_span(),
                    }),
                    span,
                ))
            }
        }
    }
}
