use std::{cmp::Ordering, iter::Peekable};

use unicode_segmentation::{Graphemes, UnicodeSegmentation};

use crate::{
    error::LexerError,
    token::{
        build::{FloatBuilder, IntBuilder, StrFormat},
        Span,
    },
    LexFilter, Token,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum IndentStyle {
    Spaces,
    Tabs,
}

pub struct LexerState {
    indent_levels: Vec<usize>,
    indent_style: Option<IndentStyle>,
    line_level: Option<usize>,
}

impl Default for LexerState {
    fn default() -> Self {
        Self {
            indent_levels: Vec::new(),
            indent_style: None,
            line_level: None,
        }
    }
}

impl LexerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_style(style: IndentStyle) -> Self {
        Self {
            indent_style: Some(style),
            ..Default::default()
        }
    }

    pub fn indent_depth(&self) -> usize {
        self.indent_levels.len()
    }

    pub fn indent_style(&self) -> Option<IndentStyle> {
        self.indent_style
    }

    pub fn close_blocks(&mut self) {
        self.line_level = Some(0);
    }

    pub fn lex<'source, 'state: 'source>(&'state mut self, source: &'source str) -> Lexer<'source> {
        Lexer {
            state: self,
            symbols: source.graphemes(true).peekable(),
            source,
            span: Span::new(0, 0),
        }
    }
}

pub struct Lexer<'source> {
    state: &'source mut LexerState,
    symbols: Peekable<Graphemes<'source>>,
    source: &'source str,
    span: Span,
}

impl<'source> Lexer<'source> {
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
        let start = self.span.start;
        Span::new(start, start)
    }

    pub fn token_end_span(&self) -> Span {
        let end = self.span.end;
        Span::new(end, end)
    }

    pub fn filtered(self) -> LexFilter<'source> {
        LexFilter::new(self)
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Result<Token<'source>, LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        // get the current line level
        let line_level = match &self.state.line_level {
            // if its already set, just use the value
            Some(level) => level,

            // otherwise calculate the new indent level
            None => {
                let mut tabs = 0;
                let mut spaces = 0;
                self.span.start = self.span.end;
                loop {
                    // peek at the next symbol
                    let Some(symbol) = self.peek_symbol() else {
                        // if its none, end the span and return none
                        self.span.start = self.span.end;
                        return None;
                    };

                    // check the symbol for indent characters
                    match symbol {
                        // TAB/SPACE CASE
                        // if a tab or space is found, then increment its value
                        " " => {
                            spaces += 1;
                            self.consume_symbol();
                        }
                        "\t" => {
                            tabs += 1;
                            self.consume_symbol();
                        }

                        // NEWLINE CASE
                        // if a newline is found, then produce a newline token
                        // empty lines should not change indent levels
                        "\r" | "\n" | "\r\n" => {
                            self.span.start = self.span.end;
                            self.consume_symbol();
                            return Some(Ok(Token::Newline));
                        }

                        // COMMENT CASE
                        // if a comment is found, then produce the comment token
                        // empty lines should not change indent levels
                        "#" => {
                            self.span.start = self.span.end;
                            self.consume_symbol();
                            return Some(Ok(self.build_comment()));
                        }

                        // END CASE
                        // if any other symbol is found, then the indent is complete
                        _ => break,
                    }
                }

                // ensure indent character match the style
                self.state.line_level.insert(match self.state.indent_style {
                    // ERROR CASES
                    Some(IndentStyle::Spaces) if tabs > 0 => {
                        return Some(Err(LexerError::MixedIndent));
                    }
                    Some(IndentStyle::Tabs) if spaces > 0 => {
                        return Some(Err(LexerError::MixedIndent));
                    }

                    // SUCCESS CASES
                    Some(IndentStyle::Tabs) => tabs,
                    Some(IndentStyle::Spaces) => spaces,
                    None => match (spaces, tabs) {
                        (0, 0) => 0,
                        (0, _) => {
                            self.state.indent_style = Some(IndentStyle::Tabs);
                            tabs
                        }
                        (_, 0) => {
                            self.state.indent_style = Some(IndentStyle::Spaces);
                            spaces
                        }
                        _ => {
                            return Some(Err(LexerError::MixedIndent));
                        }
                    },
                })
            }
        };

        // check if indent or dedent tokens need to be emitted
        let last_level = self.state.indent_levels.last().cloned().unwrap_or(0);
        match last_level.cmp(line_level) {
            // if the levels are equal, then no indent/dedent is needed
            Ordering::Equal => {}
            // if the last level is larger, then pop it and emit a dedent
            Ordering::Greater => {
                self.state.indent_levels.pop();
                return Some(Ok(Token::Dedent));
            }
            // if the last level is smaller, then push the new level and emit an indent
            Ordering::Less => {
                self.state.indent_levels.push(*line_level);
                return Some(Ok(Token::Indent));
            }
        }

        // parse other tokens
        loop {
            // peek the next symbol
            self.span.start = self.span.end;
            let Some(symbol) = self.take_symbol() else {
                // if its none, end the span and return none
                self.span.start = self.span.end;
                self.state.line_level = None;
                return None;
            };

            // match the peeked symbol
            return match symbol {
                // WHITESPACE
                // if whitespace is found, skip it and try again
                " " | "\t" => continue,

                // COMMENT CASE
                // if a comment is found, build it and return
                "#" => Some(Ok(self.build_comment())),

                // NEWLINE
                // if a newline is found, reset the line level and return a newline
                "\r" | "\n" | "\r\n" => {
                    self.state.line_level = None;
                    Some(Ok(Token::Newline))
                }

                // SINGLE SYMBOL TOKENS
                "+" => Some(Ok(Token::Add)),
                "-" => Some(Ok(Token::Sub)),
                "/" => Some(Ok(Token::Div)),
                "%" => Some(Ok(Token::Mod)),
                "." => Some(Ok(Token::Dot)),
                "," => Some(Ok(Token::Comma)),
                "?" => Some(Ok(Token::Question)),
                "(" => Some(Ok(Token::OpenParen)),
                ")" => Some(Ok(Token::CloseParen)),
                "[" => Some(Ok(Token::OpenSquare)),
                "]" => Some(Ok(Token::CloseSquare)),

                // MULTI SYMBOL TOKENS
                "*" => match self.peek_symbol() {
                    Some("*") => {
                        self.consume_symbol();
                        Some(Ok(Token::Pow))
                    }
                    _ => Some(Ok(Token::Mul)),
                },
                "=" => match self.peek_symbol() {
                    Some("=") => {
                        self.consume_symbol();
                        Some(Ok(Token::Eq))
                    }
                    _ => Some(Ok(Token::Assign)),
                },
                "!" => match self.peek_symbol() {
                    Some("=") => {
                        self.consume_symbol();
                        Some(Ok(Token::NEq))
                    }
                    _ => Some(Ok(Token::Not)),
                },
                "<" => match self.peek_symbol() {
                    Some("=") => {
                        self.consume_symbol();
                        Some(Ok(Token::LtEq))
                    }
                    _ => Some(Ok(Token::Lt)),
                },
                ">" => match self.peek_symbol() {
                    Some("=") => {
                        self.consume_symbol();
                        Some(Ok(Token::GtEq))
                    }
                    _ => Some(Ok(Token::Gt)),
                },
                "&" => match self.peek_symbol() {
                    Some("&") => {
                        self.consume_symbol();
                        Some(Ok(Token::And))
                    }
                    _ => Some(Err(LexerError::InvalidSymbol)),
                },
                "|" => match self.peek_symbol() {
                    Some("|") => {
                        self.consume_symbol();
                        Some(Ok(Token::Or))
                    }
                    _ => Some(Err(LexerError::InvalidSymbol)),
                },
                ":" => match self.peek_symbol() {
                    Some(":") => {
                        self.consume_symbol();
                        Some(Ok(Token::DoubleColon))
                    }
                    _ => Some(Ok(Token::Colon)),
                },

                // STRINGS
                quote @ "'" | quote @ "\"" => loop {
                    let Some(symbol) = self.peek_symbol() else {
                        // if a none is reached, then the string is unclosed
                        return Some(Err(LexerError::UnclosedString));
                    };

                    match symbol {
                        // if a newline is found, then the string is unclosed
                        "\r" | "\n" | "\r\n" => {
                            return Some(Err(LexerError::UnclosedString));
                        }

                        // if a matching quote is found, then end the string and return
                        _ if symbol == quote => {
                            self.consume_symbol();
                            let str_range = self.span.start + 1..self.span.end - 1;
                            let content = &self.source[str_range];
                            return Some(Ok(Token::Str(StrFormat::new(content))));
                        }

                        // if anything else is found, its just a part of the string
                        _ => self.consume_symbol(),
                    }
                },

                // IDENTIFIERS
                // if the symbol is an underscore or alphabetic, then it the token is an identifier
                symbol if symbol.chars().all(|c| c == '_' || c.is_ascii_alphabetic()) => loop {
                    match self.peek_symbol() {
                        // if the next symbol is a valid identifier symbol, consume it and continue
                        Some(symbol)
                            if symbol
                                .chars()
                                .all(|c| c == '_' || c.is_ascii_alphanumeric()) =>
                        {
                            self.consume_symbol();
                        }

                        // otherwise build the identifier
                        _ => {
                            let content = &self.source[self.span.range()];
                            break Some(Ok(match content {
                                "none" => Token::None,
                                "true" => Token::Bool(true),
                                "false" => Token::Bool(false),
                                "if" => Token::If,
                                "while" => Token::While,
                                "fn" => Token::Fn,
                                _ => Token::Ident(content),
                            }));
                        }
                    }
                },

                // NUMBERS
                // if the symbol is a number, then the token is either an int or float
                symbol if symbol.chars().all(|c| c.is_ascii_digit()) => {
                    // start trying to parse an integer
                    loop {
                        match self.peek_symbol() {
                            // if a period is found, then break and parse the float
                            Some(".") => {
                                self.consume_symbol();
                                break;
                            }

                            // if an f is found, then build and return the float early
                            Some("f") => {
                                let content = &self.source[self.span.range()];
                                self.consume_symbol(); // consume after so 'f' is not included in parsing
                                return Some(Ok(Token::Float(FloatBuilder::new(content))));
                            }

                            // if a digit is found, then consume the symbol and continue
                            Some(symbol) if symbol.chars().all(|c| c.is_ascii_digit()) => {
                                self.consume_symbol()
                            }

                            // if anything else is found, then build the integer and return
                            _ => {
                                let content = &self.source[self.span.range()];
                                return Some(Ok(Token::Int(IntBuilder::new(content))));
                            }
                        }
                    }

                    // finish parsing the float
                    loop {
                        match self.peek_symbol() {
                            // if an f is found, then we can build and return the float
                            Some("f") => {
                                let content = &self.source[self.span.range()];
                                self.consume_symbol(); // consume after so 'f' is not included in parsing
                                return Some(Ok(Token::Float(FloatBuilder::new(content))));
                            }

                            // if a digit is found, then consume the symbol and continue
                            Some(symbol) if symbol.chars().all(|c| c.is_ascii_digit()) => {
                                self.consume_symbol()
                            }

                            // if anything else is found, then build the float and return
                            _ => {
                                let content = &self.source[self.span.range()];
                                return Some(Ok(Token::Float(FloatBuilder::new(content))));
                            }
                        }
                    }
                }

                // INVALID SYMBOL
                // if any other symbol is found, then it is invalid
                _ => Some(Err(LexerError::InvalidSymbol)),
            };
        }
    }
}

// HELPER METHODS
impl<'source> Lexer<'source> {
    fn peek_symbol(&mut self) -> Option<&'source str> {
        Some(*self.symbols.peek()?)
    }

    fn take_symbol(&mut self) -> Option<&'source str> {
        let symbol = self.symbols.next()?;
        self.span.end += symbol.len();
        Some(symbol)
    }

    fn consume_symbol(&mut self) {
        if let Some(symbol) = self.symbols.next() {
            self.span.end += symbol.len();
        }
    }

    fn build_comment(&mut self) -> Token<'source> {
        while let Some(symbol) = self.peek_symbol() {
            match symbol {
                "\r" | "\n" | "\r\n" => break,
                _ => self.consume_symbol(),
            }
        }

        let content = &self.source[self.span.range()];
        Token::Comment(content)
    }
}
