use std::{cmp::Ordering, iter::Peekable};

use boba_script_parser::{core::dashu::integer::IBig, token::Span, Token};
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

use crate::{error::IndentType, LexError};

#[derive(PartialEq)]
enum TabStyle {
    Spaces,
    Tabs,
    None,
}

pub struct Lexer {
    levels: Vec<usize>,
    style: TabStyle,
    level: usize,
    indent: bool,
}

impl Lexer {
    pub fn new() -> Self {
        Self {
            levels: Vec::new(),
            style: TabStyle::None,
            level: 0,
            indent: true,
        }
    }

    pub fn close_blocks(&mut self) -> usize {
        let levels = self.levels.len();
        self.levels.clear();
        self.level = 0;
        levels
    }

    pub fn lex<'source>(&mut self, source: &'source str) -> LexTokens<'_, 'source> {
        LexTokens {
            lexer: self,
            symbols: source.graphemes(true).peekable(),
            source,
            span: Span::from(0..0),
        }
    }
}

pub struct LexTokens<'lexer, 'source> {
    lexer: &'lexer mut Lexer,
    symbols: Peekable<Graphemes<'source>>,
    source: &'source str,
    span: Span,
}

impl LexTokens<'_, '_> {
    pub fn token_start(&self) -> usize {
        self.span.start
    }

    pub fn token_end(&self) -> usize {
        self.span.end
    }

    pub fn token_span(&self) -> Span {
        (self.token_start()..self.token_end()).into()
    }
}

impl Iterator for LexTokens<'_, '_> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        // check if an indent has to be scanned
        if self.lexer.indent {
            self.lexer.indent = false;
            self.span.start = self.span.end;

            // scan all indent tokens
            let mut new_level = 0;
            loop {
                // peek the next symbol
                let Some(symbol) = self.peek_symbol() else {
                    // if there is none, consume the lin and dont update indent
                    // indent/dedent/newline tokens should not be sent on blank lines
                    self.consume_line();
                    return None;
                };

                // match the symbol with the stored indent style
                match symbol {
                    // EMPTY LINE CASE
                    // if a newline or comment is found
                    // consume the rest of the line
                    "\n" | "\r" | "\r\n" | "#" => {
                        self.consume_line();
                        return None;
                    }

                    // ARBITRARY STYLE CASES
                    // if the indent style has not been decided yet
                    // define the indent style, consume the symbol, and increment the level
                    " " if self.lexer.style == TabStyle::None => {
                        self.lexer.style = TabStyle::Spaces;
                        self.consume_symbol();
                        new_level += 1;
                    }
                    "\t" if self.lexer.style == TabStyle::None => {
                        self.lexer.style = TabStyle::Tabs;
                        self.consume_symbol();
                        new_level += 1;
                    }

                    // CORRECT STYLE CASES
                    // if the indent character matches the internal style
                    // then just consume the token and increment the level
                    " " if self.lexer.style == TabStyle::Spaces => {
                        self.consume_symbol();
                        new_level += 1;
                    }
                    "\t" if self.lexer.style == TabStyle::Tabs => {
                        self.consume_symbol();
                        new_level += 1;
                    }

                    // INVALID STYLE CASES
                    // if the indent character doesnt match the internal style, return a tab error
                    " " if self.lexer.style == TabStyle::Tabs => {
                        return self.tab_error(true);
                    }
                    "\t" if self.lexer.style == TabStyle::Spaces => {
                        return self.tab_error(false);
                    }

                    // END CASE
                    // symbol is not an indent symbol
                    _ => break,
                }
            }

            // then update the internal level
            self.lexer.level = new_level;
        }

        // check if indent/dedent tokens need to be sent
        let last_level = self.lexer.levels.last().unwrap_or(&0);
        match self.lexer.level.cmp(last_level) {
            // if the levels are equal, then do nothing and continue
            Ordering::Equal => {}
            // if the indent level is less than the last
            // then pop the last level and produce a dedent token
            Ordering::Less => {
                self.lexer.levels.pop();
                return Some(Ok(Token::Dedent));
            }
            // if the indent level is greater than the last
            // then store the new level and produce an indent token
            Ordering::Greater => {
                self.lexer.levels.push(self.lexer.level);
                return Some(Ok(Token::Indent));
            }
        }

        // if all indentation has been handled,
        // then we can move onto the rest of the regular tokens
        loop {
            // get the next symbol
            self.span.start = self.span.end;
            let Some(symbol) = self.take_symbol() else {
                self.consume_line();
                return None;
            };

            // then match the symbol to a token
            return match symbol {
                // WHITESPACE
                " " | "\t" => continue, // skip whitespace

                // NEWLINE / COMMENT
                // if a comment or newline is found, consume the line
                "\n" | "\r" | "\r\n" | "#" => {
                    self.consume_line();
                    return None;
                }

                // SIMPLE TOKENS
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
                "-" => match self.peek_symbol() {
                    Some(">") => {
                        self.consume_symbol();
                        Some(Ok(Token::Arrow))
                    }
                    _ => Some(Ok(Token::Sub)),
                },
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
                    Some(">") => {
                        self.consume_symbol();
                        Some(Ok(Token::FatArrow))
                    }
                    _ => Some(Ok(Token::Assign)),
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
                "!" => match self.peek_symbol() {
                    Some("=") => {
                        self.consume_symbol();
                        Some(Ok(Token::NEq))
                    }
                    _ => Some(Ok(Token::Not)),
                },
                ":" => match self.peek_symbol() {
                    Some("=") => {
                        self.consume_symbol();
                        Some(Ok(Token::Walrus))
                    }
                    _ => Some(Ok(Token::Colon)),
                },

                // IDENTIFIERS
                symbol if is_ident_start(symbol) => {
                    loop {
                        match self.peek_symbol() {
                            Some(symbol) if is_ident_end(symbol) => {
                                self.consume_symbol(); // consume symbol
                            }
                            _ => {
                                let ident = &self.source[self.span.range()];
                                return Some(Ok(Token::parse_ident(ident)));
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
                                let float = &self.source[self.span.range()];
                                let float = float.parse::<f64>().expect("valid float");
                                self.consume_symbol(); // take after so 'f' is not included in parsing
                                return Some(Ok(Token::Float(float)));
                            }

                            // if a digit is found then just increment the end location and continue
                            Some(symbol) if is_digit(symbol) => self.consume_symbol(),

                            // if anything else is found, then build the integer and return
                            _ => {
                                let int = &self.source[self.span.range()];
                                let int = int.parse::<IBig>().expect("valid integer");
                                return Some(Ok(Token::Int(int)));
                            }
                        }
                    }

                    // finish parsing the float
                    loop {
                        match self.peek_symbol() {
                            // if an f is found, then we can build and return the float
                            Some("f") => {
                                let float = &self.source[self.span.range()];
                                let float = float.parse::<f64>().expect("valid float");
                                self.consume_symbol(); // consume after so 'f' is not included in parsing
                                return Some(Ok(Token::Float(float)));
                            }

                            // if a digit is found then just increment the end location and continue
                            Some(symbol) if is_digit(symbol) => self.consume_symbol(),

                            // if anything else is found, then build the float and return
                            _ => {
                                let float = &self.source[self.span.range()];
                                let float = float.parse::<f64>().expect("valid float");
                                return Some(Ok(Token::Float(float)));
                            }
                        }
                    }
                }

                // STRINGS
                "'" | "\"" => loop {
                    let Some(next_symbol) = self.peek_symbol() else {
                        // if there is no symbol, then the string is unclosed
                        self.consume_line(); // consume line first
                        return Some(Err(LexError::UnclosedString));
                    };

                    match symbol {
                        // if a newline is found, then the string is unclosed
                        "\n" | "\r" | "\r\n" => {
                            self.consume_line(); // consume line first
                            return Some(Err(LexError::UnclosedString));
                        }
                        // if an escape character is found, skip the next symbol
                        "\\" => {
                            self.consume_symbol();
                            if let None = self.take_symbol() {
                                self.consume_line(); // consume line first
                                return Some(Err(LexError::UnclosedString));
                            }
                        }
                        // if a matching symbol is found, then it is the end quote
                        _ if next_symbol == symbol => {
                            self.consume_symbol();
                            let str_range = self.span.start + 1..self.span.end - 1;
                            let string = self.source[str_range].to_string();
                            return Some(Ok(Token::String(string)));
                        }
                        // otherwise the symbol is just part of the string
                        _ => self.consume_symbol(),
                    }
                },

                // INVALID SYMBOL
                _ => Some(Err(LexError::InvalidSymbol)),
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
}

// PRIVATE HELPER METHODS
impl<'source> LexTokens<'_, 'source> {
    fn consume_symbol(&mut self) {
        self.take_symbol();
    }

    fn take_symbol(&mut self) -> Option<&'source str> {
        let symbol = self.symbols.next()?;
        self.span.end += symbol.len();
        Some(symbol)
    }

    fn peek_symbol(&mut self) -> Option<&'source str> {
        Some(*self.symbols.peek()?)
    }

    fn consume_line(&mut self) {
        self.lexer.indent = true;
        self.span.start = self.span.end;
        while let Some(_) = self.symbols.next() {}
    }

    fn tab_error(&mut self, space: bool) -> Option<Result<Token, LexError>> {
        while let Some(symbol) = self.peek_symbol() {
            match symbol {
                " " | "\t" => {
                    self.consume_symbol();
                }
                "\n" | "\r" | "\r\n" | "#" => {
                    self.consume_line();
                    return None;
                }
                _ => break,
            }
        }

        match space {
            false => Some(Err(LexError::InvalidIndent(IndentType::Tab))),
            true => Some(Err(LexError::InvalidIndent(IndentType::Space))),
        }
    }
}
