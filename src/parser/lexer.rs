use std::{iter::Peekable, str::Lines};

use unic::ucd::{
    common::{is_alphanumeric, is_numeric},
    is_alphabetic,
};
use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

use crate::parser::{PError, PResult};

use super::{Span, Token};

pub type LexResult<'source> = Option<(Result<Token<'source>, &'source str>, Span)>;

pub struct TokenLines<'source> {
    lines: Peekable<Lines<'source>>,
    source: &'source str,
    start: usize,
}

impl<'source> TokenLines<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            lines: source.lines().peekable(),
            source,
            start: 0,
        }
    }

    pub fn source(&self) -> &'source str {
        self.source
    }
}

impl<'source> Iterator for TokenLines<'source> {
    type Item = TokenLine<'source>;

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.next()?; // get the next line of code

        // advance inner start index
        let start = self.start;
        self.start += line.len();

        // get parts and count leading spaces
        let mut indent = 0;
        let mut parts = line.grapheme_indices(true).peekable();
        while let Some((_, " ")) = parts.peek() {
            parts.next(); // consume space token
            indent += 1;
        }

        // return new token line
        Some(TokenLine {
            peeked: None,
            parts,
            indent,
            line,
            start,
        })
    }
}

pub struct TokenLine<'source> {
    peeked: Option<(Token<'source>, Span)>,
    parts: Peekable<GraphemeIndices<'source>>,
    line: &'source str,
    indent: usize,
    start: usize,
}

impl<'source> TokenLine<'source> {
    pub fn line(&self) -> &'source str {
        self.line
    }

    pub fn indent(&self) -> usize {
        self.indent
    }

    pub fn span(&self) -> Span {
        self.start..self.start + self.line.len()
    }

    pub fn peek(&mut self) -> Option<PResult<&(Token<'source>, Span)>> {
        if self.peeked.is_some() {
            return Some(Ok(self.peeked.as_ref().unwrap()));
        }

        self.peeked = match self.next() {
            Some(Ok(items)) => Some(items),
            Some(Err(error)) => return Some(Err(error)),
            None => None,
        };

        let peeked = self.peeked.as_ref()?;
        Some(Ok(peeked))
    }

    pub fn next_expect(&mut self, expect: impl Into<String>) -> PResult<(Token<'source>, Span)> {
        match self.next() {
            Some(result) => result,
            None => Err(PError::EndOfLine {
                expected: expect.into(),
                pos: self.start + self.line.len(),
            }),
        }
    }

    pub fn peek_expect(&mut self, expect: impl Into<String>) -> PResult<&(Token<'source>, Span)> {
        let line_end = self.start + self.line.len();
        match self.peek() {
            Some(result) => result,
            None => Err(PError::EndOfLine {
                expected: expect.into(),
                pos: line_end,
            }),
        }
    }
}

impl<'source> Iterator for TokenLine<'source> {
    type Item = PResult<(Token<'source>, Span)>;

    fn next(&mut self) -> Option<Self::Item> {
        // get the next part and index
        let (part_index, part) = match self.peeked.take() {
            // if it was already calculated, return it
            Some(items) => return Some(Ok(items)),
            None => self.parts.next()?,
        };

        // create the span start for the whole string
        let span_start = self.start + part_index;

        match part {
            // ----------
            // WHITESPACE
            " " => self.next(),

            // --------
            // COMMENTS
            "#" => {
                // consume the rest of the line and return nothing
                while let Some(_) = self.parts.next() {}
                None
            }

            // -------------
            // SINGLE TOKENS
            "+" => Some(Ok((Token::Add, span_start..span_start + part.len()))),
            "/" => Some(Ok((Token::Div, span_start..span_start + part.len()))),
            "%" => Some(Ok((Token::Mod, span_start..span_start + part.len()))),
            "?" => Some(Ok((Token::Question, span_start..span_start + part.len()))),
            "." => Some(Ok((Token::Dot, span_start..span_start + part.len()))),
            "," => Some(Ok((Token::Comma, span_start..span_start + part.len()))),
            "(" => Some(Ok((Token::OpenParen, span_start..span_start + part.len()))),
            ")" => Some(Ok((Token::CloseParen, span_start..span_start + part.len()))),

            // ------------
            // MULTI TOKENS
            "-" => match self.parts.peek() {
                Some((i, part @ ">")) => {
                    let end = i + part.len();
                    self.parts.next(); // consume peeked token
                    Some(Ok((Token::Arrow, span_start..end)))
                }
                _ => Some(Ok((Token::Sub, span_start..span_start + part.len()))),
            },
            "*" => match self.parts.peek() {
                Some((i, part @ "*")) => {
                    let end = i + part.len();
                    self.parts.next(); // consume peeked token
                    Some(Ok((Token::Pow, span_start..end)))
                }
                _ => Some(Ok((Token::Mul, span_start..span_start + part.len()))),
            },
            ":" => match self.parts.peek() {
                Some((i, part @ "=")) => {
                    let end = i + part.len();
                    self.parts.next(); // consume peeked token
                    Some(Ok((Token::Walrus, span_start..end)))
                }
                _ => Some(Ok((Token::Colon, span_start..span_start + part.len()))),
            },
            "!" => match self.parts.peek() {
                Some((i, part @ "=")) => {
                    let end = i + part.len();
                    self.parts.next(); // consume peeked token
                    Some(Ok((Token::NEq, span_start..end)))
                }
                _ => Some(Ok((Token::Not, span_start..span_start + part.len()))),
            },
            "<" => match self.parts.peek() {
                Some((i, part @ "=")) => {
                    let end = i + part.len();
                    self.parts.next(); // consume peeked token
                    Some(Ok((Token::LtEq, span_start..end)))
                }
                _ => Some(Ok((Token::Lt, span_start..span_start + part.len()))),
            },
            ">" => match self.parts.peek() {
                Some((i, part @ "=")) => {
                    let end = i + part.len();
                    self.parts.next(); // consume peeked token
                    Some(Ok((Token::GtEq, span_start..end)))
                }
                _ => Some(Ok((Token::Gt, span_start..span_start + part.len()))),
            },
            "=" => match self.parts.peek() {
                Some((i, part @ "=")) => {
                    let end = i + part.len();
                    self.parts.next(); // consume peeked token
                    Some(Ok((Token::Eq, span_start..end)))
                }
                _ => Some(Ok((Token::Assign, span_start..span_start + part.len()))),
            },

            // -------
            // STRINGS
            quote @ "'" | quote @ "\"" => loop {
                match self.parts.peek() {
                    Some((quote_index, part)) if *part == quote => {
                        let str_span = part_index + quote.len()..*quote_index;
                        let span = span_start + quote.len()..self.start + quote_index;
                        self.parts.next(); // consume token
                        return Some(Ok((Token::String(&self.line[str_span]), span)));
                    }
                    None => {
                        let span = span_start..self.start + self.line.len();
                        return Some(Err(PError::UnclosedString {
                            quote: quote.into(),
                            span,
                        }));
                    }
                    Some(_) => {
                        self.parts.next(); // consume token
                        continue;
                    }
                }
            },

            // --------------------
            // IDENTIFIERS/KEYWORDS
            part if is_ident_start(part) => loop {
                match self.parts.peek() {
                    Some((_, part)) if is_ident_mid(part) => {
                        self.parts.next(); // consume token
                        continue;
                    }
                    Some((ident_end, _)) => {
                        let str = &self.line[part_index..*ident_end];
                        let span = span_start..self.start + ident_end;

                        // check for keywords
                        return match Token::get_keyword(str) {
                            Some(token) => Some(Ok((token, span))),
                            None => Some(Ok((Token::Ident(str), span))),
                        };
                    }
                    None => {
                        let str = &self.line[part_index..];
                        let span = span_start..self.start + self.line.len();

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
            part if is_number(part) => {
                loop {
                    match self.parts.peek() {
                        Some((_, part)) if is_number(part) => {
                            self.parts.next(); // consume number
                            continue;
                        }
                        Some((_, ".")) => {
                            self.parts.next();
                            break; // break and continue parsing float
                        }
                        Some((int_end, _)) => {
                            let str = &self.line[part_index..*int_end];
                            let span = span_start..self.start + int_end;
                            return Some(Ok((Token::UInt(str), span)));
                        }
                        None => {
                            let str = &self.line[part_index..];
                            let span = span_start..self.start + self.line.len();
                            return Some(Ok((Token::UInt(str), span)));
                        }
                    }
                }

                // after here we are parsing floats
                loop {
                    match self.parts.peek() {
                        Some((_, part)) if is_number(part) => {
                            self.parts.next(); // consume number
                            continue;
                        }
                        Some((float_end, _)) => {
                            let str = &self.line[part_index..*float_end];
                            let span = span_start..self.start + float_end;
                            return Some(Ok((Token::UFloat(str), span)));
                        }
                        None => {
                            let str = &self.line[part_index..];
                            let span = span_start..self.start + self.line.len();
                            return Some(Ok((Token::UFloat(str), span)));
                        }
                    }
                }
            }

            // --------------
            // INVALID TOKENS
            part => {
                let span = span_start..span_start + part.len();
                Some(Err(PError::InvalidToken {
                    part: part.into(),
                    span,
                }))
            }
        }
    }
}

fn is_ident_start(str: &str) -> bool {
    str.chars().all(|c| c == '_' || is_alphabetic(c))
}

fn is_ident_mid(str: &str) -> bool {
    str.chars().all(|c| c == '_' || is_alphanumeric(c))
}

fn is_number(str: &str) -> bool {
    str.chars().all(is_numeric)
}
