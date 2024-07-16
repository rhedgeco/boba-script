use std::io;

use boba_script::{
    core::{engine::Value, Engine},
    parser::{
        parsers::statement::{self, StatementParser, StatementType},
        TokenLine,
    },
};
use boba_script_ariadne::ToAriadne;
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
    pending: StatementParser<ShellSource>,
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
            pending: StatementParser::none(),
        }
    }
}

impl Shell {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_line(&mut self) -> io::Result<RunState> {
        // choose a prompt
        let prompt = match self.pending.is_none() {
            false => &self.pending_prompt,
            true => &self.normal_prompt,
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

        loop {
            // get the next line of tokens
            let mut line = TokenLine::new(&mut self.tokens);

            // get pending or create the next statement
            let statement = match self.pending.is_none() {
                false => match self.pending.parse_line(&mut line) {
                    Err(errors) => Err(errors),
                    Ok(Some(statement)) => Ok(statement),
                    Ok(None) => match self.tokens.is_empty() {
                        false => continue,
                        true => break,
                    },
                },
                true => match statement::start_parsing(&mut line) {
                    Err(errors) => Err(errors),
                    Ok(StatementType::SingleLine(statement)) => Ok(statement),
                    Ok(StatementType::MultiLine(parser)) => {
                        self.pending = parser;
                        match self.tokens.is_empty() {
                            false => continue,
                            true => break,
                        }
                    }
                },
            };

            // execute the completed statement
            match statement {
                Ok(statement) => match self.engine.eval(statement) {
                    Ok(Value::None) => {} // do nothing
                    Ok(value) => println!("{value}"),
                    Err(error) => error
                        .to_ariadne()
                        .eprint(self.tokens.build_cache())
                        .unwrap(),
                },
                Err(errors) => {
                    let mut cache = self.tokens.build_cache();
                    for error in errors {
                        error.to_ariadne().eprint(&mut cache).unwrap();
                    }
                }
            }

            // break if there are no more tokens
            if self.tokens.is_empty() {
                break;
            }
        }

        Ok(RunState::Parsed)
    }
}
