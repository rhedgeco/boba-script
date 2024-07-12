use std::io;

use boba_script::{
    core::{engine::Value, Engine},
    lexer::LexError,
    parser::{
        parsers::statement::{self, ParseState, StatementParser},
        TokenLine,
    },
};
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

use super::{stream::ShellSource, ShellStream};

pub enum RunState {
    Parsed,
    CtrlD,
    CtrlC,
}

pub struct Shell {
    editor: Reedline,
    normal_prompt: DefaultPrompt,
    pending_prompt: DefaultPrompt,
    tokens: ShellStream,
    engine: Engine<ShellSource>,
    pending: Option<StatementParser<ShellSource, LexError>>,
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
            tokens: ShellStream::new(),
            engine: Engine::new(),
            pending: None,
        }
    }
}

impl Shell {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_line(&mut self) -> io::Result<RunState> {
        // choose a prompt
        let prompt = match self.pending.is_some() {
            true => &self.pending_prompt,
            false => &self.normal_prompt,
        };

        // get the text
        let text = match self.editor.read_line(prompt)? {
            Signal::Success(text) => text,
            Signal::CtrlC => {
                return Ok(RunState::CtrlC);
            }
            Signal::CtrlD => {
                return Ok(RunState::CtrlD);
            }
        };

        // load the tokens
        self.tokens.load(text);

        // keep parsing while there are still tokens
        while !self.tokens.is_empty() {
            // get the next line of tokens
            let mut line = TokenLine::new(&mut self.tokens);

            // get pending or create the next statement
            let statement = match self.pending.take() {
                Some(parser) => match parser.parse_line(&mut line) {
                    Err(errors) => Err(errors),
                    Ok(ParseState::Complete(statement)) => Ok(statement),
                    Ok(ParseState::Incomplete(parser)) => {
                        self.pending = Some(parser);
                        continue;
                    }
                },
                None => match statement::start_parsing(&mut line) {
                    Err(errors) => Err(errors),
                    Ok(ParseState::Complete(statement)) => Ok(statement),
                    Ok(ParseState::Incomplete(parser)) => {
                        self.pending = Some(parser);
                        continue;
                    }
                },
            };

            // execute the completed statement
            match statement {
                Ok(statement) => match self.engine.eval(statement) {
                    Ok(Value::None) => {} // do nothing
                    Ok(value) => println!("{value}"),
                    Err(error) => eprintln!("{error:?}"),
                },
                Err(errors) => {
                    for error in errors {
                        eprintln!("{error:?}");
                    }
                }
            }
        }

        Ok(RunState::Parsed)
    }
}
