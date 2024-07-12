use std::{cmp::Ordering, iter::Peekable};

use boba_script_parser::{core::dashu::integer::IBig, token::Span, Token};
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

use crate::{error::IndentType, LexerError, TextLine};

#[derive(PartialEq)]
enum TabStyle {
    Spaces,
    Tabs,
    None,
}

pub enum LexResult {
    Token(Token),
    Error(LexerError),
    Incomplete,
    Finished,
}

pub struct LexerState {
    // indent management
    pending_block: bool,
    levels: Vec<usize>,
    style: TabStyle,
    level: usize,
    indent: bool,

    // span management
    progress: usize,
    start: usize,
}

impl LexerState {
    pub fn new() -> Self {
        Self {
            pending_block: false,
            levels: Vec::new(),
            style: TabStyle::None,
            level: 0,
            indent: true,
            progress: 0,
            start: 0,
        }
    }

    pub fn close_all_blocks(&mut self) {
        self.pending_block = false;
        self.levels.clear();
        self.level = 0;
    }

    pub fn is_finished(&self) -> bool {
        if !self.levels.is_empty() {
            return false;
        }

        if self.pending_block {
            return false;
        }

        true
    }
}

pub struct LineLexer<'a> {
    symbols: Peekable<Graphemes<'a>>,
    line: &'a str,
    state: LexerState,
    span: Span,
}

impl<'a> LineLexer<'a> {
    pub fn new(line: TextLine<'a>) -> Self {
        Self::new_with(line, LexerState::new())
    }

    pub fn new_with(line: TextLine<'a>, state: LexerState) -> Self {
        Self {
            symbols: line.text().graphemes(true).peekable(),
            line: line.text(),
            state,
            span: (0..0).into(),
        }
    }

    pub fn consume(self) -> LexerState {
        self.state
    }

    pub fn span(&self) -> Span {
        let start = self.state.start + self.span.start;
        let end = self.state.start + self.span.end;
        (start..end).into()
    }

    pub fn generate(&mut self) -> LexResult {
        // check if an indent has to be scanned
        if self.state.indent {
            self.state.indent = false;
            self.span.start = self.span.end;

            // scan all indent tokens
            let mut new_level = 0;
            loop {
                // peek the next symbol
                let Some(symbol) = self.peek_symbol() else {
                    // if there is none, consume the lin and dont update indent
                    // indent/dedent/newline tokens should not be sent on blank lines
                    return self.consume_line();
                };

                // match the symbol with the stored indent style
                match symbol {
                    // EMPTY LINE CASE
                    // if a newline or comment is found
                    // consume the rest of the line
                    "\n" | "\r" | "\r\n" | "#" => {
                        return self.consume_line();
                    }

                    // ARBITRARY STYLE CASES
                    // if the indent style has not been decided yet
                    // define the indent style, consume the symbol, and increment the level
                    " " if self.state.style == TabStyle::None => {
                        self.state.style = TabStyle::Spaces;
                        self.consume_symbol();
                        new_level += 1;
                    }
                    "\t" if self.state.style == TabStyle::None => {
                        self.state.style = TabStyle::Tabs;
                        self.consume_symbol();
                        new_level += 1;
                    }

                    // CORRECT STYLE CASES
                    // if the indent character matches the internal style
                    // then just consume the token and increment the level
                    " " if self.state.style == TabStyle::Spaces => {
                        self.consume_symbol();
                        new_level += 1;
                    }
                    "\t" if self.state.style == TabStyle::Tabs => {
                        self.consume_symbol();
                        new_level += 1;
                    }

                    // INVALID STYLE CASES
                    // if the indent character doesnt match the internal style, return a tab error
                    " " if self.state.style == TabStyle::Tabs => {
                        return self.tab_error(true);
                    }
                    "\t" if self.state.style == TabStyle::Spaces => {
                        return self.tab_error(false);
                    }

                    // END CASE
                    // symbol is not an indent symbol
                    _ => break,
                }
            }

            // then update the internal level
            self.state.level = new_level;
        }

        // check if indent/dedent tokens need to be sent
        let last_level = self.state.levels.last().unwrap_or(&0);
        match self.state.level.cmp(last_level) {
            // if the levels are equal, then do nothing and continue
            Ordering::Equal => {}
            // if the indent level is less than the last
            // then pop the last level and produce a dedent token
            Ordering::Less => {
                self.state.levels.pop();
                return LexResult::Token(Token::Dedent);
            }
            // if the indent level is greater than the last
            // then store the new level and produce an indent token
            Ordering::Greater => {
                self.state.levels.push(self.state.level);
                return LexResult::Token(Token::Indent);
            }
        }

        // if all indentation has been handled,
        // then we can move onto the rest of the regular tokens
        loop {
            // get the next symbol
            self.span.start = self.span.end;
            let Some(symbol) = self.take_symbol() else {
                return self.consume_line();
            };

            self.state.pending_block = false;
            return match symbol {
                // WHITESPACE
                " " | "\t" => continue, // skip whitespace

                // NEWLINE / COMMENT
                // if a comment or newline is found, consume the line
                "\n" | "\r" | "\r\n" | "#" => {
                    return self.consume_line();
                }

                // SIMPLE TOKENS
                "+" => LexResult::Token(Token::Add),
                "/" => LexResult::Token(Token::Div),
                "%" => LexResult::Token(Token::Modulo),
                "." => LexResult::Token(Token::Period),
                "," => LexResult::Token(Token::Comma),
                ";" => LexResult::Token(Token::SemiColon),
                "?" => LexResult::Token(Token::Question),
                "(" => LexResult::Token(Token::OpenParen),
                ")" => LexResult::Token(Token::CloseParen),
                "{" => LexResult::Token(Token::OpenCurly),
                "}" => LexResult::Token(Token::CloseCurly),
                "[" => LexResult::Token(Token::OpenSquare),
                "]" => LexResult::Token(Token::CloseSquare),

                // MULTI TOKENS
                "-" => match self.peek_symbol() {
                    Some(">") => {
                        self.consume_symbol();
                        LexResult::Token(Token::Arrow)
                    }
                    _ => LexResult::Token(Token::Sub),
                },
                "*" => match self.peek_symbol() {
                    Some("*") => {
                        self.consume_symbol();
                        LexResult::Token(Token::Pow)
                    }
                    _ => LexResult::Token(Token::Mul),
                },
                "=" => match self.peek_symbol() {
                    Some("=") => {
                        self.consume_symbol();
                        LexResult::Token(Token::Eq)
                    }
                    Some(">") => {
                        self.consume_symbol();
                        LexResult::Token(Token::FatArrow)
                    }
                    _ => LexResult::Token(Token::Assign),
                },
                "<" => match self.peek_symbol() {
                    Some("=") => {
                        self.consume_symbol();
                        LexResult::Token(Token::LtEq)
                    }
                    _ => LexResult::Token(Token::Lt),
                },
                ">" => match self.peek_symbol() {
                    Some("=") => {
                        self.consume_symbol();
                        LexResult::Token(Token::GtEq)
                    }
                    _ => LexResult::Token(Token::Gt),
                },
                "!" => match self.peek_symbol() {
                    Some("=") => {
                        self.consume_symbol();
                        LexResult::Token(Token::NEq)
                    }
                    _ => LexResult::Token(Token::Not),
                },
                ":" => match self.peek_symbol() {
                    Some("=") => {
                        self.consume_symbol();
                        LexResult::Token(Token::Walrus)
                    }
                    _ => {
                        self.state.pending_block = true;
                        LexResult::Token(Token::Colon)
                    }
                },

                // IDENTIFIERS
                symbol if is_ident_start(symbol) => {
                    loop {
                        match self.peek_symbol() {
                            Some(symbol) if is_ident_end(symbol) => {
                                self.consume_symbol(); // consume symbol
                            }
                            _ => {
                                let ident = &self.line[self.span.range()];
                                return LexResult::Token(Token::parse_ident(ident));
                            }
                        }
                    }
                }

                // NUMBERS
                symbol if is_digit(symbol) => {
                    // start parsing an integer
                    loop {
                        match self.peek_symbol() {
                            // if a period is found then we can break and parse the float
                            Some(".") => {
                                self.consume_symbol();
                                break;
                            }

                            // if an f is found, then we can build and return the float early
                            Some("f") => {
                                let float = &self.line[self.span.range()];
                                let float = float.parse::<f64>().expect("valid float");
                                self.consume_symbol(); // take after so 'f' is not included in parsing
                                return LexResult::Token(Token::Float(float));
                            }

                            // if a digit is found then just increment the end location and continue
                            Some(symbol) if is_digit(symbol) => self.consume_symbol(),

                            // if anything else is found, then build the integer and return
                            _ => {
                                let int = &self.line[self.span.range()];
                                let int = int.parse::<IBig>().expect("valid integer");
                                return LexResult::Token(Token::Int(int));
                            }
                        }
                    }

                    // finish parsing the float
                    loop {
                        match self.peek_symbol() {
                            // if an f is found, then we can build and return the float
                            Some("f") => {
                                let float = &self.line[self.span.range()];
                                let float = float.parse::<f64>().expect("valid float");
                                self.consume_symbol(); // consume after so 'f' is not included in parsing
                                return LexResult::Token(Token::Float(float));
                            }

                            // if a digit is found then just increment the end location and continue
                            Some(symbol) if is_digit(symbol) => self.consume_symbol(),

                            // if anything else is found, then build the float and return
                            _ => {
                                let float = &self.line[self.span.range()];
                                let float = float.parse::<f64>().expect("valid float");
                                return LexResult::Token(Token::Float(float));
                            }
                        }
                    }
                }

                // STRINGS
                "'" | "\"" => loop {
                    let Some(next_symbol) = self.peek_symbol() else {
                        // if there is no symbol, then the string is unclosed
                        self.consume_line(); // consume line first
                        return LexResult::Error(LexerError::UnclosedString);
                    };

                    match symbol {
                        // if a newline is found, then the string is unclosed
                        "\n" | "\r" | "\r\n" => {
                            self.consume_line(); // consume line first
                            return LexResult::Error(LexerError::UnclosedString);
                        }
                        // if an escape character is found, skip the next symbol
                        "\\" => {
                            self.consume_symbol();
                            if let None = self.take_symbol() {
                                self.consume_line(); // consume line first
                                return LexResult::Error(LexerError::UnclosedString);
                            }
                        }
                        // if a matching symbol is found, then it is the end quote
                        _ if next_symbol == symbol => {
                            self.consume_symbol();
                            let str_range = self.span.start + 1..self.span.end - 1;
                            let string = self.line[str_range].to_string();
                            return LexResult::Token(Token::String(string));
                        }
                        // otherwise the symbol is just part of the string
                        _ => self.consume_symbol(),
                    }
                },

                // INVALID SYMBOL
                _ => LexResult::Error(LexerError::InvalidSymbol),
            };
        }

        // HELPER FUNCTIONS
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

    fn consume_symbol(&mut self) {
        self.take_symbol();
    }

    fn take_symbol(&mut self) -> Option<&'a str> {
        let symbol = self.symbols.next()?;
        self.state.progress += symbol.len();
        self.span.end += symbol.len();
        Some(symbol)
    }

    fn peek_symbol(&mut self) -> Option<&'a str> {
        Some(*self.symbols.peek()?)
    }

    fn tab_error(&mut self, space: bool) -> LexResult {
        while let Some(symbol) = self.peek_symbol() {
            match symbol {
                " " | "\t" => {
                    self.take_symbol();
                }
                "\n" | "\r" | "\r\n" | "#" => {
                    return self.consume_line();
                }
                _ => break,
            }
        }

        match space {
            false => LexResult::Error(LexerError::InvalidIndent(IndentType::Tab)),
            true => LexResult::Error(LexerError::InvalidIndent(IndentType::Space)),
        }
    }

    fn consume_line(&mut self) -> LexResult {
        self.state.indent = true;
        self.span.start = self.span.end;
        while let Some(symbol) = self.symbols.next() {
            self.state.progress += symbol.len();
        }
        match self.state.is_finished() {
            false => return LexResult::Incomplete,
            true => return LexResult::Finished,
        }
    }
}
