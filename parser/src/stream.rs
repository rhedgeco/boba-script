use crate::{error::ParseError, token::Span, PError, Token};

pub trait SpanSource: Clone + Sized {
    fn span(&self) -> Span;
    fn start(&self) -> usize;
    fn end(&self) -> usize;
    fn build(&self, span: impl Into<Span>) -> Self;
}

impl<T: SpanSource> SpanSourceExt for T {}
pub trait SpanSourceExt: SpanSource {
    fn start_source(&self) -> Self {
        let start = self.start();
        self.build(start..start)
    }

    fn end_source(&self) -> Self {
        let end = self.end();
        self.build(end..end)
    }
}

pub trait TokenStream: Iterator<Item = Result<Token, Self::Error>> + Sized {
    type Error;
    type Source: SpanSource;
    fn token_start(&self) -> usize;
    fn token_end(&self) -> usize;
    fn build_source(&self, span: impl Into<Span>) -> Self::Source;
}

impl<T: TokenStream> TokenStream for &mut T {
    type Error = T::Error;
    type Source = T::Source;

    fn token_start(&self) -> usize {
        T::token_start(self)
    }

    fn token_end(&self) -> usize {
        T::token_end(self)
    }

    fn build_source(&self, span: impl Into<Span>) -> Self::Source {
        T::build_source(self, span)
    }
}

// blanket token stream extension trait
impl<T: TokenStream> StreamExt for T {}
pub trait StreamExt: TokenStream {
    fn token_span(&self) -> Span {
        Span::from(self.token_start()..self.token_end())
    }

    fn token_source(&self) -> Self::Source {
        self.build_source(self.token_span())
    }
}

pub struct LineParser<T: TokenStream> {
    peeked: Option<Token>,
    span: Span,
    stream: T,
}

impl<T: TokenStream> LineParser<T> {
    pub fn new(stream: T) -> Self {
        let end = stream.token_end();
        Self {
            peeked: None,
            span: Span::from(end..end),
            stream,
        }
    }

    pub fn line(&mut self) -> TokenLine<T> {
        TokenLine { parser: self }
    }

    pub fn token_start(&self) -> usize {
        self.span.start
    }

    pub fn token_end(&self) -> usize {
        self.span.end
    }

    pub fn token_source(&self) -> T::Source {
        self.stream.build_source(self.span)
    }

    pub fn token_end_source(&self) -> T::Source {
        let end = self.span.end;
        self.stream.build_source(end..end)
    }

    pub fn token_start_source(&self) -> T::Source {
        let start = self.span.start;
        self.stream.build_source(start..start)
    }

    pub fn build_source(&self, span: impl Into<Span>) -> T::Source {
        self.stream.build_source(span)
    }

    pub fn consume_line(&mut self) -> Result<(), Vec<PError<T>>> {
        let mut errors = match self.take_next() {
            Err(error) => vec![error],
            Ok(Some(Token::Newline)) | Ok(None) => return Ok(()),
            Ok(Some(token)) => vec![ParseError::UnexpectedInput {
                expect: "end of line".into(),
                found: Some(token),
                source: self.line().token_source(),
            }],
        };

        loop {
            // keep consuming until a newline is found
            match self.take_next() {
                Err(error) => errors.push(error),
                Ok(Some(Token::Newline)) | Ok(None) => return Err(errors),
                _ => continue,
            }
        }
    }

    fn take_next(&mut self) -> Result<Option<Token>, PError<T>> {
        // try to take the peeked token first
        if let Some(token) = self.peeked.take() {
            self.span = self.stream.token_span();
            return Ok(Some(token));
        }

        // then generate the next one
        match self.stream.next() {
            None => {
                let end = self.span.end;
                self.span = Span::from(end..end);
                Ok(None)
            }
            Some(Ok(token)) => {
                self.span = self.stream.token_span();
                Ok(Some(token))
            }
            Some(Err(error)) => {
                self.span = self.stream.token_span();
                let span = self.stream.token_source();
                Err(ParseError::TokenError {
                    error,
                    source: span,
                })
            }
        }
    }

    fn peek_next(&mut self) -> Result<Option<&Token>, PError<T>> {
        match &self.peeked {
            // try to look inside peeked
            Some(_) => match &self.peeked {
                Some(token) => Ok(Some(token)),
                _ => unreachable!(),
            },
            // otherwise generate a new token and store it
            None => match self.stream.next() {
                None => Ok(None),
                Some(Ok(token)) => Ok(Some(self.peeked.insert(token))),
                // if an error is found, just return it
                // errors are not stored so they wont be produced twice
                Some(Err(error)) => Err(ParseError::TokenError {
                    source: self.stream.token_source(),
                    error,
                }),
            },
        }
    }
}

pub struct TokenLine<'a, T: TokenStream> {
    parser: &'a mut LineParser<T>,
}

impl<'a, T: TokenStream> TokenLine<'a, T> {
    pub fn token_start(&self) -> usize {
        self.parser.token_start()
    }

    pub fn token_end(&self) -> usize {
        self.parser.token_end()
    }

    pub fn token_source(&self) -> T::Source {
        self.parser.token_source()
    }

    pub fn token_end_source(&self) -> T::Source {
        self.parser.token_end_source()
    }

    pub fn token_start_source(&self) -> T::Source {
        self.parser.token_start_source()
    }

    pub fn build_source(&self, span: impl Into<Span>) -> T::Source {
        self.parser.build_source(span)
    }

    pub fn take_next(&mut self) -> Result<Option<Token>, PError<T>> {
        match self.parser.take_next()? {
            Some(Token::Newline) => {
                self.parser.peeked = Some(Token::Newline);
                Ok(None)
            }
            token => Ok(token),
        }
    }

    pub fn peek_next(&mut self) -> Result<Option<&Token>, PError<T>> {
        match self.parser.peek_next()? {
            Some(Token::Newline) => Ok(None),
            token => Ok(token),
        }
    }

    pub fn take_expect(&mut self, expect: Option<&Token>) -> Result<(), PError<T>> {
        let token = self.take_next()?;
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

    pub fn peek_expect(&mut self, expect: Option<&Token>) -> Result<(), PError<T>> {
        let token = self.peek_next()?;
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

    pub fn parse_else<O>(
        &mut self,
        parse: impl FnOnce(&mut TokenLine<T>) -> Result<O, Vec<PError<T>>>,
        on_error: impl FnOnce(&mut ErrorParser<T>),
    ) -> Result<O, Vec<PError<T>>> {
        match parse(&mut self.parser.line()) {
            Ok(output) => Ok(output),
            Err(errors) => {
                let mut parser = ErrorParser {
                    line: self.parser.line(),
                    errors,
                };
                on_error(&mut parser);
                Err(parser.errors)
            }
        }
    }

    pub fn parse_next<O>(
        &mut self,
        validate: impl FnOnce(Option<Token>, &mut TokenLine<T>) -> Result<O, Vec<PError<T>>>,
    ) -> Result<O, Vec<PError<T>>> {
        self.parse_next_else(validate, |_| {})
    }

    pub fn parse_peek<O>(
        &mut self,
        validate: impl FnOnce(PeekParser<T>) -> Result<O, Vec<PError<T>>>,
    ) -> Result<O, Vec<PError<T>>> {
        self.parse_peek_else(validate, |_| {})
    }

    pub fn parse_next_else<O>(
        &mut self,
        validate: impl FnOnce(Option<Token>, &mut TokenLine<T>) -> Result<O, Vec<PError<T>>>,
        on_error: impl FnOnce(&mut ErrorParser<T>),
    ) -> Result<O, Vec<PError<T>>> {
        self.parse_else(
            |line| {
                let token = line.take_next().map_err(|e| vec![e])?;
                validate(token, &mut line.parser.line())
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
            |line| {
                line.peek_next().map_err(|e| vec![e])?;
                validate(PeekParser {
                    line: line.parser.line(),
                })
            },
            on_error,
        )
    }

    pub fn consume_line(&mut self, store: &mut Vec<PError<T>>) {
        loop {
            match self.take_next() {
                Ok(Some(_)) => {}
                Ok(None) => return,
                Err(error) => store.push(error),
            }
        }
    }

    pub fn consume_until_inclusive(
        &mut self,
        store: &mut Vec<PError<T>>,
        inclusive: impl Fn(&Token) -> bool,
    ) -> Option<Token> {
        loop {
            match self.take_next() {
                Ok(None) => return None,
                Err(error) => store.push(error),
                Ok(Some(token)) => {
                    if inclusive(&token) {
                        return Some(token);
                    }
                }
            }
        }
    }

    pub fn consume_until_exclusive(
        &mut self,
        store: &mut Vec<PError<T>>,
        exclusive: impl Fn(&Token) -> bool,
    ) -> Option<&Token> {
        loop {
            match self.peek_next() {
                Ok(None) => return None,
                Err(error) => store.push(error),
                Ok(Some(token)) => {
                    if exclusive(&token) {
                        match self.peek_next() {
                            Ok(Some(token)) => return Some(token),
                            _ => unreachable!(),
                        }
                    }

                    // consume and continue if it was not exclusive
                    match self.take_next() {
                        Ok(Some(_)) => continue,
                        _ => unreachable!(),
                    }
                }
            }
        }
    }

    pub fn consume_until(
        &mut self,
        store: &mut Vec<PError<T>>,
        inclusive: impl Fn(&Token) -> bool,
        exclusive: impl Fn(&Token) -> bool,
    ) -> UntilKind {
        loop {
            match self.peek_next() {
                // if we find none, return none
                Ok(None) => return UntilKind::None,

                // if an error is found, store it
                Err(error) => store.push(error),

                // if a token is found, test it
                Ok(Some(token)) => {
                    // check if token is inclusive
                    if inclusive(token) {
                        match self.take_next() {
                            Ok(Some(token)) => return UntilKind::Inclusive(token),
                            _ => unreachable!(),
                        }
                    }

                    // then check if it is exclusive
                    if exclusive(token) {
                        match self.peek_next() {
                            Ok(Some(token)) => return UntilKind::Exclusive(token),
                            _ => unreachable!(),
                        }
                    }

                    // consume and continue if it was not exclusive
                    match self.take_next() {
                        Ok(Some(_)) => continue,
                        _ => unreachable!(),
                    }
                }
            }
        }
    }
}

pub enum UntilKind<'a> {
    Inclusive(Token),
    Exclusive(&'a Token),
    None,
}

pub struct PeekParser<'a, T: TokenStream> {
    line: TokenLine<'a, T>,
}

impl<'a, T: TokenStream> PeekParser<'a, T> {
    pub fn token(&self) -> Option<&Token> {
        Some(self.line.parser.peeked.as_ref()?)
    }

    pub fn token_source(&self) -> T::Source {
        match &self.line.parser.peeked {
            Some(_) => self.line.parser.stream.token_source(),
            None => self.line.token_end_source(),
        }
    }

    pub fn ignore(self) -> TokenLine<'a, T> {
        self.line
    }

    pub fn consume(self) -> TokenLine<'a, T> {
        self.take().1
    }

    pub fn take(self) -> (Option<Token>, TokenLine<'a, T>) {
        match self.line.parser.peeked.take() {
            None => (None, self.line),
            Some(Token::Newline) => {
                self.line.parser.peeked = Some(Token::Newline);
                (None, self.line)
            }
            Some(token) => {
                self.line.parser.span = self.line.parser.stream.token_span();
                (Some(token), self.line)
            }
        }
    }
}

pub struct ErrorParser<'a, T: TokenStream> {
    errors: Vec<PError<T>>,
    line: TokenLine<'a, T>,
}

impl<'a, T: TokenStream> ErrorParser<'a, T> {
    pub fn token_start(&self) -> usize {
        self.line.token_start()
    }

    pub fn token_end(&self) -> usize {
        self.line.token_end()
    }

    pub fn token_source(&self) -> T::Source {
        self.line.token_source()
    }

    pub fn token_end_source(&self) -> T::Source {
        self.line.token_end_source()
    }

    pub fn token_start_source(&self) -> T::Source {
        self.line.token_start_source()
    }

    pub fn build_source(&self, span: impl Into<Span>) -> T::Source {
        self.line.build_source(span)
    }

    pub fn push(&mut self, error: PError<T>) {
        self.errors.push(error)
    }

    pub fn consume_line(&mut self) {
        self.line.consume_line(&mut self.errors)
    }

    pub fn consume_until_inclusive(&mut self, inclusive: impl Fn(&Token) -> bool) -> Option<Token> {
        self.line
            .consume_until_inclusive(&mut self.errors, inclusive)
    }

    pub fn consume_until_exclusive(
        &mut self,
        exclusive: impl Fn(&Token) -> bool,
    ) -> Option<&Token> {
        self.line
            .consume_until_exclusive(&mut self.errors, exclusive)
    }

    pub fn consume_until(
        &mut self,
        inclusive: impl Fn(&Token) -> bool,
        exclusive: impl Fn(&Token) -> bool,
    ) -> UntilKind {
        self.line
            .consume_until(&mut self.errors, exclusive, inclusive)
    }
}
