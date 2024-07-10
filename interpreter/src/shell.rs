use std::{cmp::Ordering, io};

use boba_script::{
    core::{ast::StatementNode, engine::Value, Engine},
    lexer::{cache::CacheSpan, BobaCache, Lexer, LexerError},
    parser::{
        error::ParseError,
        parsers::statement::{self, Header, IncompleteStatement},
        stream::StreamExt,
        Token,
    },
};
use boba_script_ariadne::ToAriadne;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

pub enum RunState {
    Success,
    CtrlD,
    CtrlC,
}

struct Pending {
    indent: usize,
    statement: IncompleteStatement<CacheSpan>,
    body: Vec<StatementNode<CacheSpan>>,
}

impl Pending {
    pub fn new(indent: usize, statement: IncompleteStatement<CacheSpan>) -> Self {
        Self {
            indent,
            statement,
            body: Vec::new(),
        }
    }

    pub fn finish(self) -> Result<StatementNode<CacheSpan>, ParseError<CacheSpan, LexerError>> {
        self.statement.finish_with(self.body)
    }
}

pub struct Shell {
    editor: Reedline,
    normal_prompt: DefaultPrompt,
    pending_prompt: DefaultPrompt,
    engine: Engine<CacheSpan>,
    cache: BobaCache,
    pending: Vec<Pending>,
}

impl Default for Shell {
    fn default() -> Self {
        Self {
            editor: Reedline::create(),
            normal_prompt: DefaultPrompt::new(
                DefaultPromptSegment::Basic(format!("boba ")),
                DefaultPromptSegment::Empty,
            ),
            pending_prompt: DefaultPrompt::new(
                DefaultPromptSegment::Basic(format!("  ...")),
                DefaultPromptSegment::Empty,
            ),
            engine: Default::default(),
            cache: Default::default(),
            pending: Vec::new(),
        }
    }
}

impl Shell {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_line(&mut self) -> io::Result<RunState> {
        let prompt = match self.pending.is_empty() {
            false => &self.pending_prompt,
            true => &self.normal_prompt,
        };

        let data = match self.editor.read_line(prompt)? {
            Signal::Success(buffer) => self.cache.store("shell", buffer),
            Signal::CtrlD => {
                return Ok(RunState::CtrlD);
            }
            Signal::CtrlC => {
                return Ok(RunState::CtrlC);
            }
        };

        let mut parser = Lexer::new(data).parser();
        let indent = match parser.line().peek_next() {
            Ok(None) => {
                if let Some(pending) = self.pending.pop() {
                    match pending.finish() {
                        Err(error) => error.to_ariadne().eprint(&mut self.cache)?,
                        Ok(statement) => match self.engine.eval(statement) {
                            Ok(Value::None) => {}
                            Ok(value) => println!("{value}"),
                            Err(error) => error.to_ariadne().eprint(&mut self.cache)?,
                        },
                    }
                }

                return Ok(RunState::Success);
            }
            Err(error) => {
                error.to_ariadne().eprint(&mut self.cache)?;
                return Ok(RunState::Success);
            }
            Ok(Some(Token::Indent)) => match parser.line().take_next() {
                Ok(Some(Token::Indent)) => parser.stream().indent_level(),
                _ => unreachable!(),
            },
            Ok(_) => 0,
        };

        match statement::parse_header(&mut parser.line()) {
            // if the statement is incomplete, push it to the stack read the next line
            Ok(Header::Incomplete(statement)) => {
                self.pending.push(Pending::new(indent, statement));
                return Ok(self.read_line()?);
            }

            // if its complete, check the pending stack
            Ok(Header::Complete(statement)) => loop {
                match self.pending.pop() {
                    // if the pending stack has items, try to complete it
                    Some(mut pending) => match indent.cmp(&pending.indent) {
                        // if the indent greater than the pending indent then the statement is part of it
                        // push the statement into its body, re-queue the pending item and break
                        Ordering::Greater => {
                            pending.body.push(statement);
                            self.pending.push(pending);
                            break;
                        }

                        // if the indent is equal or less than the pending indent
                        // then complete and execute this pending item and continue
                        _ => match pending.finish() {
                            Err(error) => error.to_ariadne().eprint(&mut self.cache)?,
                            Ok(statement) => match self.engine.eval(statement) {
                                Ok(Value::None) => {}
                                Ok(value) => println!("{value}"),
                                Err(error) => error.to_ariadne().eprint(&mut self.cache)?,
                            },
                        },
                    },

                    // if the stack is empty
                    None => {
                        // execute and print the statement
                        match self.engine.eval(statement) {
                            Ok(Value::None) => {}
                            Ok(value) => println!("{value}"),
                            Err(error) => error.to_ariadne().eprint(&mut self.cache)?,
                        }

                        // and break out of the loop
                        break;
                    }
                }
            },
            // otherwise print the errors
            Err(errors) => {
                for error in errors {
                    error.to_ariadne().eprint(&mut self.cache)?;
                }
            }
        }

        Ok(RunState::Success)
    }
}
