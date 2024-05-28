use std::{iter::Peekable, ops::Range};

use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

use crate::{
    cache::{CacheData, CacheSpan},
    parser::{PError, PResult, Token},
};

pub struct Lexer<'source> {
    peeked: Option<(Token<'source>, CacheSpan)>,
    symbols: Peekable<GraphemeIndices<'source>>,
    data: &'source CacheData,
    pos: usize,
}

impl<'source> Lexer<'source> {
    pub fn new(data: &'source CacheData) -> Self {
        Self {
            peeked: None,
            symbols: data.text().grapheme_indices(true).peekable(),
            data,
            pos: 0,
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn pos_span(&self) -> CacheSpan {
        self.data.span(self.pos..self.pos)
    }

    pub fn span(&self, range: Range<usize>) -> CacheSpan {
        self.data.span(range)
    }

    pub fn peek(&mut self) -> Option<PResult<CacheSpan, (Token<'source>, CacheSpan)>> {
        match &self.peeked {
            Some(items) => Some(Ok(items.clone())),
            None => match self.next()? {
                Err(err) => Some(Err(err)),
                Ok(items) => {
                    self.peeked = Some(items.clone());
                    Some(Ok(items))
                }
            },
        }
    }

    pub fn expect_peek(
        &mut self,
        expect: impl Into<String>,
    ) -> PResult<CacheSpan, (Token<'source>, CacheSpan)> {
        match self.peek() {
            Some(result) => result,
            None => Err(PError::UnexpectedEnd {
                expected: expect.into(),
                data: self.pos_span(),
            }),
        }
    }

    pub fn expect_next(
        &mut self,
        expect: impl Into<String>,
    ) -> PResult<CacheSpan, (Token<'source>, CacheSpan)> {
        match self.next() {
            Some(result) => result,
            None => Err(PError::UnexpectedEnd {
                expected: expect.into(),
                data: self.pos_span(),
            }),
        }
    }

    pub fn expect_line_end(&mut self) -> PResult<CacheSpan, ()> {
        match self.next() {
            None => Ok(()),
            Some(Err(err)) => Err(err),
            Some(Ok((token, span))) => match token {
                Token::Newline => Ok(()),
                _ => Err(PError::UnexpectedToken {
                    expected: format!("end of line"),
                    found: format!("'{token}'"),
                    data: span,
                }),
            },
        }
    }

    fn peek_symbol(&mut self) -> Option<(usize, &'source str)> {
        let (index, symbol) = self.symbols.peek()?;
        Some((*index, *symbol))
    }

    fn take_symbol(&mut self) -> Option<(usize, &'source str)> {
        let (index, symbol) = self.symbols.next()?;
        self.pos = index + symbol.len();
        Some((index, symbol))
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = PResult<CacheSpan, (Token<'source>, CacheSpan)>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(peeked) = self.peeked.take() {
            self.pos = peeked.1.range().end;
            return Some(Ok(peeked));
        }

        loop {
            let (start, symbol) = self.take_symbol()?;
            let symbol_span = self.data.span(start..start + symbol.len());
            self.pos = start + symbol.len(); // update scan pos

            // --------------
            // HELPER METHODS
            fn is_newline(str: &str) -> bool {
                str == "\n" || str == "\r\n" || str == "\r"
            }

            fn is_digits(str: &str) -> bool {
                str.chars().all(|c| c.is_ascii_digit())
            }

            fn is_ident_start(str: &str) -> bool {
                str.chars().all(|c| c == '_' || c.is_ascii_alphabetic())
            }

            fn is_ident_end(str: &str) -> bool {
                str.chars().all(|c| c == '_' || c.is_ascii_alphanumeric())
            }

            return match symbol {
                // ----------
                // WHITESPACE
                " " | "\t" => continue,

                // --------
                // NEWLINES
                symbol if is_newline(symbol) => Some(Ok((Token::Newline, symbol_span))),

                // --------
                // COMMENTS
                "#" => {
                    // check if token is some and not a newline
                    while self.peek_symbol().is_some_and(|(_, s)| !is_newline(s)) {
                        // consume the tokens until end of line is reached
                        self.take_symbol();
                    }

                    // continue so that entire comment is skipped by lexer
                    continue;
                }

                // -------------
                // SINGLE TOKENS
                "+" => Some(Ok((Token::Add, symbol_span))),
                "/" => Some(Ok((Token::Div, symbol_span))),
                "%" => Some(Ok((Token::Mod, symbol_span))),
                "?" => Some(Ok((Token::Question, symbol_span))),
                "." => Some(Ok((Token::Dot, symbol_span))),
                "," => Some(Ok((Token::Comma, symbol_span))),
                "(" => Some(Ok((Token::OpenParen, symbol_span))),
                ")" => Some(Ok((Token::CloseParen, symbol_span))),

                // ------------
                // MULTI TOKENS
                "-" => match self.peek_symbol() {
                    Some((index, part @ ">")) => {
                        let end = index + part.len();
                        self.take_symbol(); // consume peeked token
                        Some(Ok((Token::Arrow, self.data.span(start..end))))
                    }
                    _ => Some(Ok((Token::Sub, symbol_span))),
                },
                "*" => match self.peek_symbol() {
                    Some((index, part @ "*")) => {
                        let end = index + part.len();
                        self.take_symbol(); // consume peeked token
                        Some(Ok((Token::Pow, self.data.span(start..end))))
                    }
                    _ => Some(Ok((Token::Mul, symbol_span))),
                },
                "!" => match self.peek_symbol() {
                    Some((index, part @ "=")) => {
                        let end = index + part.len();
                        self.take_symbol(); // consume peeked token
                        Some(Ok((Token::NEq, self.data.span(start..end))))
                    }
                    _ => Some(Ok((Token::Not, symbol_span))),
                },
                "<" => match self.peek_symbol() {
                    Some((index, part @ "=")) => {
                        let end = index + part.len();
                        self.take_symbol(); // consume peeked token
                        Some(Ok((Token::LtEq, self.data.span(start..end))))
                    }
                    _ => Some(Ok((Token::Lt, symbol_span))),
                },
                ">" => match self.peek_symbol() {
                    Some((index, part @ "=")) => {
                        let end = index + part.len();
                        self.take_symbol(); // consume peeked token
                        Some(Ok((Token::GtEq, self.data.span(start..end))))
                    }
                    _ => Some(Ok((Token::Gt, symbol_span))),
                },
                "=" => match self.peek_symbol() {
                    Some((index, part @ "=")) => {
                        let end = index + part.len();
                        self.take_symbol(); // consume peeked token
                        Some(Ok((Token::Eq, self.data.span(start..end))))
                    }
                    _ => Some(Ok((Token::Assign, symbol_span))),
                },
                ":" => match self.peek_symbol() {
                    Some((index, part @ "=")) => {
                        let end = index + part.len();
                        self.take_symbol(); // consume peeked token
                        Some(Ok((Token::Walrus, self.data.span(start..end))))
                    }
                    _ => Some(Ok((Token::Colon, symbol_span))),
                },

                // -------
                // STRINGS
                quote @ "'" | quote @ "\"" => loop {
                    match self.peek_symbol() {
                        Some((index, symbol)) if symbol == quote => {
                            let str_range = start + quote.len()..index;
                            let data_range = start..index + quote.len();
                            let span = self.data.span(data_range);
                            self.take_symbol(); // consume token

                            let str = &self.data.text()[str_range];
                            return Some(Ok((Token::String(str), span)));
                        }
                        None => {
                            let span = start..self.data.text().len();
                            return Some(Err(PError::UnclosedString {
                                data: self.data.span(span),
                            }));
                        }
                        Some(_) => {
                            self.take_symbol(); // consume token
                            continue;
                        }
                    }
                },

                // --------------------
                // IDENTIFIERS/KEYWORDS
                symbol if is_ident_start(symbol) => loop {
                    match self.peek_symbol() {
                        Some((_, symbol)) if is_ident_end(symbol) => {
                            self.take_symbol(); // consume token
                            continue;
                        }
                        Some((end, _)) => {
                            let span = self.data.span(start..end);
                            let str = &self.data.text()[span.range().clone()];

                            // check for keywords
                            return match Token::get_keyword(str) {
                                Some(token) => Some(Ok((token, span))),
                                None => Some(Ok((Token::Ident(str), span))),
                            };
                        }
                        None => {
                            let span = self.data.span(start..self.data.text().len());
                            let str = &self.data.text()[span.range().clone()];

                            // check for keywords
                            return match Token::get_keyword(str) {
                                Some(token) => Some(Ok((token, span))),
                                None => Some(Ok((Token::Ident(str), span))),
                            };
                        }
                    }
                },

                // -------
                // NUMBERS
                symbol if is_digits(symbol) => {
                    loop {
                        match self.peek_symbol() {
                            Some((_, symbol)) if is_digits(symbol) => {
                                self.take_symbol(); // consume number
                                continue;
                            }
                            Some((_, ".")) => {
                                self.take_symbol();
                                break; // break and continue parsing float
                            }
                            Some((end, _)) => {
                                let span = self.data.span(start..end);
                                let str = &self.data.text()[span.range().clone()];
                                return Some(Ok((Token::UInt(str), span)));
                            }
                            None => {
                                let span = self.data.span(start..self.data.text().len());
                                let str = &self.data.text()[span.range().clone()];
                                return Some(Ok((Token::UInt(str), span)));
                            }
                        }
                    }

                    // after here we are parsing floats
                    loop {
                        match self.peek_symbol() {
                            Some((_, symbol)) if is_digits(symbol) => {
                                self.take_symbol(); // consume number
                                continue;
                            }
                            Some((end, _)) => {
                                let span = self.data.span(start..end);
                                let str = &self.data.text()[span.range().clone()];
                                return Some(Ok((Token::UFloat(str), span)));
                            }
                            None => {
                                let span = self.data.span(start..self.data.text().len());
                                let str = &self.data.text()[span.range().clone()];
                                return Some(Ok((Token::UFloat(str), span)));
                            }
                        }
                    }
                }

                // --------------
                // INVALID TOKENS
                symbol => Some(Err(PError::InvalidToken {
                    part: symbol.into(),
                    data: symbol_span,
                })),
            };
        }
    }
}
