use std::{iter::Peekable, ops::Range, str::SplitInclusive};

use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

use crate::{
    cache::{CacheData, Span},
    parser::{PError, PResult, Token},
};

pub enum TokenItem {}

pub struct Lexer<'source> {
    data: &'source CacheData,
    lines: SplitInclusive<'source, char>,
    tabs: Option<bool>,
    offset: usize,
}

impl<'source> Lexer<'source> {
    pub fn new(data: &'source CacheData) -> Self {
        Self {
            data,
            lines: data.text().split_inclusive('\n'),
            tabs: None,
            offset: 0,
        }
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = PResult<TokenLine<'source>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // get line and ensure it is not empty
            let line = self.lines.next()?;
            if line.trim().is_empty() {
                continue;
            }

            // count indentation and validate tabs and spaces
            let mut indent = 0;
            for (byte, char) in line.chars().enumerate() {
                match char {
                    ' ' => match self.tabs {
                        Some(false) => indent += 1,
                        None => {
                            self.tabs = Some(false);
                            indent += 1;
                        }
                        Some(true) => {
                            let range = self.offset + byte..self.offset + byte + char.len_utf8();
                            let span = self.data.span(range);
                            return Some(Err(PError::MixedTabsAndSpaces { span, tab: false }));
                        }
                    },
                    '\t' => match self.tabs {
                        Some(true) => indent += 1,
                        None => {
                            self.tabs = Some(true);
                            indent += 1;
                        }
                        Some(false) => {
                            let range = self.offset + byte..self.offset + byte + char.len_utf8();
                            let span = self.data.span(range);
                            return Some(Err(PError::MixedTabsAndSpaces { span, tab: true }));
                        }
                    },
                    _ => break,
                }
            }

            // calculate data range and increment internal offset
            let range = self.offset..self.offset + line.len();
            self.offset += line.len();

            // build and return token line
            return Some(Ok(TokenLine {
                peeked: None,
                symbols: line.grapheme_indices(true).peekable(),
                data: self.data,
                range,
                indent,
            }));
        }
    }
}

pub struct TokenLine<'source> {
    peeked: Option<(Token<'source>, Span)>,
    symbols: Peekable<GraphemeIndices<'source>>,
    data: &'source CacheData,
    range: Range<usize>,
    indent: usize,
}

impl<'source> TokenLine<'source> {
    pub fn indent(&self) -> usize {
        self.indent
    }

    pub fn line(&self) -> &'source str {
        &self.data.text()[self.range.clone()]
    }

    pub fn span(&self, range: Range<usize>) -> Span {
        self.data.span(range)
    }

    pub fn peek(&mut self) -> Option<PResult<&(Token<'source>, Span)>> {
        if self.peeked.is_some() {
            return self.peeked.as_ref().map(|p| Ok(p));
        }

        self.peeked = match self.next()? {
            Err(e) => return Some(Err(e)),
            Ok(peek) => Some(peek),
        };

        self.peeked.as_ref().map(|p| Ok(p))
    }

    pub fn expect_end(&mut self) -> PResult<()> {
        match self.next() {
            None => Ok(()),
            Some(Err(e)) => Err(e),
            Some(Ok((token, span))) => Err(PError::UnexpectedToken {
                expected: format!("end of line"),
                found: format!("'{token}'"),
                span,
            }),
        }
    }

    pub fn expect_next(&mut self, expect: impl Into<String>) -> PResult<(Token<'source>, Span)> {
        match self.next() {
            Some(next) => next,
            None => Err(PError::UnexpectedEndOfLine {
                expected: expect.into(),
                span: self.data.span(self.range.end..self.range.end),
            }),
        }
    }

    pub fn expect_peek(&mut self, expect: impl Into<String>) -> PResult<&(Token<'source>, Span)> {
        let span = self.data.span(self.range.end..self.range.end);
        match self.peek() {
            Some(peek) => peek,
            None => Err(PError::UnexpectedEndOfLine {
                expected: expect.into(),
                span,
            }),
        }
    }
}

impl<'source> Iterator for TokenLine<'source> {
    type Item = PResult<(Token<'source>, Span)>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(peeked) = self.peeked.take() {
            return Some(Ok(peeked));
        }

        // --------------
        // HELPER METHODS
        fn is_whitespace(str: &str) -> bool {
            str.chars().all(|c| c.is_whitespace())
        }

        fn valid_digits(str: &str) -> bool {
            str.chars().all(|c| c.is_ascii_digit())
        }

        fn valid_ident_start(str: &str) -> bool {
            str.chars().all(|c| c == '_' || c.is_ascii_alphabetic())
        }

        fn valid_ident_end(str: &str) -> bool {
            str.chars().all(|c| c == '_' || c.is_ascii_alphanumeric())
        }

        // ---------
        // MAIN LOOP
        loop {
            let (offset, symbol) = self.symbols.next()?;
            let offset = self.range.start + offset;
            let symbol_span = self.data.span(offset..offset + symbol.len());
            return match symbol {
                // ----------
                // WHITESPACE
                symbol if is_whitespace(symbol) => continue,

                // --------
                // COMMENTS
                "#" => {
                    // consume the rest of the line and return nothing
                    while let Some(_) = self.symbols.next() {}
                    None
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
                "-" => match self.symbols.peek() {
                    Some((i, part @ ">")) => {
                        let end = offset + i + part.len();
                        self.symbols.next(); // consume peeked token
                        Some(Ok((Token::Arrow, self.data.span(offset..end))))
                    }
                    _ => Some(Ok((Token::Sub, symbol_span))),
                },
                "*" => match self.symbols.peek() {
                    Some((i, part @ "*")) => {
                        let end = offset + i + part.len();
                        self.symbols.next(); // consume peeked token
                        Some(Ok((Token::Pow, self.data.span(offset..end))))
                    }
                    _ => Some(Ok((Token::Mul, symbol_span))),
                },
                "!" => match self.symbols.peek() {
                    Some((i, part @ "=")) => {
                        let end = offset + i + part.len();
                        self.symbols.next(); // consume peeked token
                        Some(Ok((Token::NEq, self.data.span(offset..end))))
                    }
                    _ => Some(Ok((Token::Not, symbol_span))),
                },
                "<" => match self.symbols.peek() {
                    Some((i, part @ "=")) => {
                        let end = offset + i + part.len();
                        self.symbols.next(); // consume peeked token
                        Some(Ok((Token::LtEq, self.data.span(offset..end))))
                    }
                    _ => Some(Ok((Token::Lt, symbol_span))),
                },
                ">" => match self.symbols.peek() {
                    Some((i, part @ "=")) => {
                        let end = offset + i + part.len();
                        self.symbols.next(); // consume peeked token
                        Some(Ok((Token::GtEq, self.data.span(offset..end))))
                    }
                    _ => Some(Ok((Token::Gt, symbol_span))),
                },
                "=" => match self.symbols.peek() {
                    Some((i, part @ "=")) => {
                        let end = offset + i + part.len();
                        self.symbols.next(); // consume peeked token
                        Some(Ok((Token::Eq, self.data.span(offset..end))))
                    }
                    _ => Some(Ok((Token::Assign, symbol_span))),
                },
                ":" => match self.symbols.peek() {
                    Some((i, part @ "=")) => {
                        let end = offset + i + part.len();
                        self.symbols.next(); // consume peeked token
                        Some(Ok((Token::Walrus, self.data.span(offset..end))))
                    }
                    _ => Some(Ok((Token::Colon, symbol_span))),
                },

                // -------
                // STRINGS
                quote @ "'" | quote @ "\"" => loop {
                    match self.symbols.peek() {
                        Some((quote_index, symbol)) if *symbol == quote => {
                            let str_range = offset + quote.len()..*quote_index;
                            let data_range = offset..self.range.start + quote_index + quote.len();
                            let span = self.data.span(data_range);
                            self.symbols.next(); // consume token

                            let str = &self.data.text()[str_range];
                            return Some(Ok((Token::String(str), span)));
                        }
                        None => {
                            let span = offset..self.range.end;
                            return Some(Err(PError::UnclosedString {
                                span: self.data.span(span),
                            }));
                        }
                        Some(_) => {
                            self.symbols.next(); // consume token
                            continue;
                        }
                    }
                },

                // --------------------
                // IDENTIFIERS/KEYWORDS
                symbol if valid_ident_start(symbol) => loop {
                    match self.symbols.peek() {
                        Some((_, symbol)) if valid_ident_end(symbol) => {
                            self.symbols.next(); // consume token
                            continue;
                        }
                        Some((ident_end, _)) => {
                            let span = self.data.span(offset..self.range.start + ident_end);
                            let str = &self.data.text()[span.range().clone()];

                            // check for keywords
                            return match Token::get_keyword(str) {
                                Some(token) => Some(Ok((token, span))),
                                None => Some(Ok((Token::Ident(str), span))),
                            };
                        }
                        None => {
                            let span = self.data.span(offset..self.range.end);
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
                symbol if valid_digits(symbol) => {
                    loop {
                        match self.symbols.peek() {
                            Some((_, symbol)) if valid_digits(symbol) => {
                                self.symbols.next(); // consume number
                                continue;
                            }
                            Some((_, ".")) => {
                                self.symbols.next();
                                break; // break and continue parsing float
                            }
                            Some((int_end, _)) => {
                                let span = self.data.span(offset..self.range.start + int_end);
                                let str = &self.data.text()[span.range().clone()];
                                return Some(Ok((Token::UInt(str), span)));
                            }
                            None => {
                                let span = self.data.span(offset..self.range.end);
                                let str = &self.data.text()[span.range().clone()];
                                return Some(Ok((Token::UInt(str), span)));
                            }
                        }
                    }

                    // after here we are parsing floats
                    loop {
                        match self.symbols.peek() {
                            Some((_, symbol)) if valid_digits(symbol) => {
                                self.symbols.next(); // consume number
                                continue;
                            }
                            Some((float_end, _)) => {
                                let span = self.data.span(offset..self.range.start + float_end);
                                let str = &self.data.text()[span.range().clone()];
                                return Some(Ok((Token::UFloat(str), span)));
                            }
                            None => {
                                let span = self.data.span(offset..self.range.end);
                                let str = &self.data.text()[span.range().clone()];
                                return Some(Ok((Token::UFloat(str), span)));
                            }
                        }
                    }
                }

                // --------------
                // INVALID TOKENS
                symbol => {
                    let span = offset..offset + symbol.len();
                    Some(Err(PError::InvalidToken {
                        part: symbol.into(),
                        span: self.data.span(span),
                    }))
                }
            };
        }
    }
}
