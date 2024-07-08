use std::{cmp::Ordering, iter::Peekable};

use boba_script_parser::{
    core::dashu::integer::IBig, stream::Source, token::Span, Token, TokenStream,
};
use unicode_segmentation::{GraphemeIndices, UnicodeSegmentation};

use crate::{cache::CacheData, error::IndentType, LexerError};

#[derive(Debug, PartialEq)]
enum IndentStyle {
    Spaces,
    Tabs,
    None,
}

pub struct Lexer<'a> {
    pending_error: Option<(LexerError, Span)>,
    symbols: Peekable<GraphemeIndices<'a>>,
    cache: &'a CacheData,
    span: Span,

    // indent tracking data
    pending_indent: Option<(usize, Span)>,
    indent_levels: Vec<usize>,
    indent_style: IndentStyle,
}

impl<'a> TokenStream for Lexer<'a> {
    type Error = LexerError;
    type Source = CacheData;

    fn span(&self) -> Span {
        self.span
    }

    fn source(&self) -> &Self::Source {
        self.cache
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        // first return any pending errors
        if let Some((error, span)) = self.pending_error.take() {
            self.span = span;
            return Some(Err(error));
        }

        // then return any pending indent/dedent tokens
        if let Some((new_level, span)) = self.pending_indent.take() {
            let last_level = self.indent_levels.last().unwrap_or(&0);
            match new_level.cmp(last_level) {
                // if the new level is equal to the last
                // then no action is required and we can move on
                Ordering::Equal => {}
                // if the new level is greater than the last
                // then we need to store the new level and emit an indent token
                Ordering::Greater => {
                    self.span = span;
                    self.indent_levels.push(new_level);
                    return Some(Ok(Token::Indent));
                }
                // if the new level is less than the last
                // then we need to pop the last level, reset the pending level, and emit a dedent token
                Ordering::Less => {
                    self.span = span;
                    self.indent_levels.pop();
                    self.pending_indent = Some((new_level, span));
                    return Some(Ok(Token::Dedent));
                }
            }
        }

        // finally generate any other tokens
        loop {
            let (start, symbol) = self.symbols.next()?;
            self.span = Span::from(start..start + symbol.len());
            return match symbol {
                // WHITESPACE
                " " | "\t" => continue, // skip whitespace

                // COMMENTS
                "#" => {
                    // consume all symbols until a newline is found
                    self.consume_until(|s| matches!(s, "\n" | "\r" | "\r\n"));
                    continue; // then skip to the next token
                }

                // NEWLINES
                "\n" | "\r" | "\r\n" => {
                    self.capture_indent();
                    Some(Ok(Token::Newline))
                }

                // SINGLE TOKENS
                "+" => Some(Ok(Token::Add)),
                "/" => Some(Ok(Token::Div)),
                "%" => Some(Ok(Token::Modulo)),
                "." => Some(Ok(Token::Period)),
                "," => Some(Ok(Token::Comma)),
                ";" => Some(Ok(Token::SemiColon)),
                "?" => Some(Ok(Token::Question)),
                "(" => Some(Ok(Token::OpenParen)),
                ")" => Some(Ok(Token::CloseParen)),
                "{" => Some(Ok(Token::OpenCurly)),
                "}" => Some(Ok(Token::CloseCurly)),
                "[" => Some(Ok(Token::OpenSquare)),
                "]" => Some(Ok(Token::CloseSquare)),

                // MULTI TOKENS
                "-" => match self.symbols.peek() {
                    Some((_, symbol @ ">")) => {
                        self.span.end += symbol.len();
                        Some(Ok(Token::Arrow))
                    }
                    Some(_) | None => Some(Ok(Token::Sub)),
                },
                "*" => match self.symbols.peek() {
                    Some((_, symbol @ "*")) => {
                        self.span.end += symbol.len();
                        Some(Ok(Token::Pow))
                    }
                    Some(_) | None => Some(Ok(Token::Mul)),
                },
                "=" => match self.symbols.peek() {
                    Some((_, symbol @ "=")) => {
                        self.span.end += symbol.len();
                        Some(Ok(Token::Eq))
                    }
                    Some((_, symbol @ ">")) => {
                        self.span.end += symbol.len();
                        Some(Ok(Token::FatArrow))
                    }
                    Some(_) | None => Some(Ok(Token::Assign)),
                },
                "<" => match self.symbols.peek() {
                    Some((_, symbol @ "=")) => {
                        self.span.end += symbol.len();
                        Some(Ok(Token::LtEq))
                    }
                    Some(_) | None => Some(Ok(Token::Lt)),
                },
                ">" => match self.symbols.peek() {
                    Some((_, symbol @ "=")) => {
                        self.span.end += symbol.len();
                        Some(Ok(Token::GtEq))
                    }
                    Some(_) | None => Some(Ok(Token::Gt)),
                },
                "!" => match self.symbols.peek() {
                    Some((_, symbol @ "=")) => {
                        self.span.end += symbol.len();
                        Some(Ok(Token::NEq))
                    }
                    Some(_) | None => Some(Ok(Token::Not)),
                },
                ":" => match self.symbols.peek() {
                    Some((_, symbol @ "=")) => {
                        self.span.end += symbol.len();
                        Some(Ok(Token::Walrus))
                    }
                    Some(_) | None => Some(Ok(Token::Colon)),
                },

                // IDENTIFIERS
                symbol if is_ident_start(&symbol) => {
                    loop {
                        match self.symbols.peek() {
                            Some((_, symbol)) if is_ident_end(symbol) => {
                                self.span.end += symbol.len();
                                self.symbols.next(); // consume symbol
                            }
                            Some(_) | None => {
                                let ident = &self.cache.text()[self.span.range()];
                                return Some(Ok(Token::get_ident_or_keyword(ident)));
                            }
                        }
                    }
                }

                // NUMBERS
                symbol if is_digit(symbol) => {
                    // start parsing an integer
                    loop {
                        match self.symbols.peek() {
                            // if a period is found then we can break and parse the float
                            Some((_, ".")) => {
                                self.symbols.next();
                                self.span.end += symbol.len();
                                break;
                            }
                            // if an f is found, then we can build and return the float early
                            Some((_, "f")) => {
                                self.symbols.next();
                                let float = &self.cache.text()[self.span.range()];
                                self.span.end += 1; // increment the token span to contain the f
                                let float = float.parse::<f64>().expect("valid float");
                                return Some(Ok(Token::Float(float)));
                            }
                            // if a digit is found then just increment the end location and continue
                            Some((_, symbol)) if is_digit(symbol) => {
                                self.span.end += symbol.len();
                                self.symbols.next();
                            }
                            // if anything else is found, then build the integer and return
                            Some(_) | None => {
                                let int = &self.cache.text()[self.span.range()];
                                let int = int.parse::<IBig>().expect("valid integer");
                                return Some(Ok(Token::Int(int)));
                            }
                        }
                    }

                    // finish parsing the float
                    loop {
                        match self.symbols.peek() {
                            // if an f is found, then we can build and return the float
                            Some((_, "f")) => {
                                self.symbols.next();
                                let float = &self.cache.text()[self.span.range()];
                                self.span.end += 1; // increment the token span to contain the f
                                let float = float.parse::<f64>().expect("valid float");
                                return Some(Ok(Token::Float(float)));
                            }
                            // if a digit is found then just increment the end location and continue
                            Some((_, symbol)) if is_digit(symbol) => {
                                self.span.end += symbol.len();
                                self.symbols.next();
                            }
                            // if anything else is found, then build the float and return
                            Some(_) | None => {
                                let float = &self.cache.text()[self.span.range()];
                                let float = float.parse::<f64>().expect("valid float");
                                return Some(Ok(Token::Float(float)));
                            }
                        }
                    }
                }

                // STRINGS
                "'" | "\"" => loop {
                    let next_symbol = match self.symbols.peek() {
                        None => return Some(Err(LexerError::UnclosedString)),
                        Some((_, symbol)) => *symbol,
                    };

                    match symbol {
                        // if a newline is found, then the string is unclosed
                        "\n" | "\r" | "\r\n" => return Some(Err(LexerError::UnclosedString)),
                        // if an escape character is found, skip the next symbol
                        "\\" => {
                            self.span.end += next_symbol.len();
                            self.symbols.next();
                            match self.symbols.next() {
                                None => return Some(Err(LexerError::UnclosedString)),
                                Some((_, symbol)) => self.span.end += symbol.len(),
                            }
                        }
                        // if a matching symbol is found, then it is the end quote
                        _ if next_symbol == symbol => {
                            self.symbols.next(); // consume end quote
                            self.span.end += next_symbol.len();
                            let str_range = self.span.start + 1..self.span.end - 1;
                            let string = self.cache.text()[str_range].to_string();
                            return Some(Ok(Token::String(string)));
                        }
                        // otherwise the symbol is just part of the string
                        _ => {
                            self.span.end += next_symbol.len();
                            self.symbols.next(); // consume token
                        }
                    }
                },

                // FAILURE
                symbol => {
                    self.span = Span::from(start..start + symbol.len());
                    Some(Err(LexerError::InvalidSymbol))
                }
            };

            // HELPER METHODS
            fn is_ident_start(s: &str) -> bool {
                s.chars().all(|c| c == '_' || c.is_ascii_alphabetic())
            }

            fn is_ident_end(s: &str) -> bool {
                s.chars().all(|c| c == '_' || c.is_ascii_alphanumeric())
            }

            fn is_digit(s: &str) -> bool {
                s.chars().all(|c| c.is_ascii_digit())
            }
        }
    }
}

impl<'a> Lexer<'a> {
    pub fn new(data: &'a CacheData) -> Self {
        // create the lexer
        let mut lexer = Self {
            pending_error: None,
            symbols: data.text().grapheme_indices(true).peekable(),
            span: Span::from(0..0),
            cache: data,

            pending_indent: None,
            indent_style: IndentStyle::None,
            indent_levels: Vec::new(),
        };

        // capture the initial indent and return the new lexer
        lexer.capture_indent();
        lexer
    }

    pub fn indent_level(&self) -> usize {
        *self.indent_levels.last().unwrap_or(&0)
    }

    fn consume_until(&mut self, until: impl Fn(&str) -> bool) {
        loop {
            match self.symbols.peek() {
                Some((_, str)) => match until(*str) {
                    // if the correct token is not found, consume and try again
                    false => {
                        self.symbols.next();
                    }
                    // if it is found then just return
                    true => return,
                },
                // if there are no tokens left, just return
                None => return,
            }
        }
    }

    fn capture_indent(&mut self) {
        // first capture the start of the indentation
        let mut indent_start = match self.symbols.peek() {
            Some((indent_start, _)) => *indent_start,
            None => return,
        };

        // then count how many indent tokens there are
        let mut indent_level = 0;
        loop {
            let (indent_end, symbol) = match self.symbols.peek() {
                Some((indent_end, symbol)) => (*indent_end, *symbol),
                None => return,
            };

            match symbol {
                // EMPTY LINE CASE
                // if a newline is found, consume the newline, reset the counter and continue
                "\n" | "\r" | "\r\n" => {
                    indent_start = indent_end + symbol.len();
                    self.symbols.next(); // consume newline
                    indent_level = 0;
                }

                // COMMENT CASE
                // if a comment is found, consume the comment and continue
                "#" => self.consume_until(|s| matches!(s, "\n" | "\r" | "\r\n")),

                // INCREMENT CASES
                // increment the indent level when the symbol matches the indent style
                " " if self.indent_style == IndentStyle::None => {
                    self.indent_style = IndentStyle::Spaces;
                    self.symbols.next(); // consume space
                    indent_level += 1;
                }
                "\t" if self.indent_style == IndentStyle::None => {
                    self.indent_style = IndentStyle::Tabs;
                    self.symbols.next(); // consume tab
                    indent_level += 1;
                }
                " " if self.indent_style == IndentStyle::Spaces => {
                    self.symbols.next(); // consume space
                    indent_level += 1;
                }
                "\t" if self.indent_style == IndentStyle::Tabs => {
                    self.symbols.next(); // consume tab
                    indent_level += 1;
                }

                // FAILURE CASES
                // store an error when the symbol does not match the indent style
                " " if self.indent_style == IndentStyle::Tabs => {
                    // check the rest of the indent tokens to ensure indent is valid
                    let indent_end = loop {
                        let (indent_end, symbol) = match self.symbols.peek() {
                            Some(items) => items,
                            None => return,
                        };
                        match *symbol {
                            " " | "\t" => self.symbols.next(),
                            "\n" | "\r" | "\r\n" | "#" => return,
                            _ => break *indent_end,
                        };
                    };

                    // then store the indentation error and return true
                    self.pending_error = Some((
                        LexerError::InvalidIndent(IndentType::Space),
                        Span::from(indent_start..indent_end),
                    ));
                    return;
                }
                "\t" if self.indent_style == IndentStyle::Spaces => {
                    // check the rest of the indent tokens to ensure indent is valid
                    let indent_end = loop {
                        let (indent_end, symbol) = match self.symbols.peek() {
                            Some(items) => items,
                            None => return,
                        };
                        match *symbol {
                            " " | "\t" => self.symbols.next(),
                            "\n" | "\r" | "\r\n" | "#" => return,
                            _ => break *indent_end,
                        };
                    };

                    // then store the indentation error and return true
                    self.pending_error = Some((
                        LexerError::InvalidIndent(IndentType::Tab),
                        Span::from(indent_start..indent_end),
                    ));
                    return;
                }

                // END CASES
                // when any other token is found, the indent is assumed to be ended
                // and we can store the indent level and return true
                _ => {
                    let indent_span = Span::from(indent_start..indent_end);
                    self.pending_indent = Some((indent_level, indent_span));
                    return;
                }
            }
        }
    }
}
