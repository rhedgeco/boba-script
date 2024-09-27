use std::io;

use boba_script::{
    engine::{scope::LocalScope, Eval, Value},
    lexer::LexerState,
    parser::{grammar, spanned::SpannedLexer},
};
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

pub enum RunState {
    Parsed,
    CtrlD,
    CtrlC,
}

pub struct Shell {
    scope: LocalScope,
    editor: Reedline,
    normal_prompt: DefaultPrompt,
    lexer: LexerState,
}

impl Default for Shell {
    fn default() -> Self {
        Self {
            scope: LocalScope::default(),
            editor: Reedline::create(),
            normal_prompt: DefaultPrompt::new(
                DefaultPromptSegment::Basic(format!("boba ")),
                DefaultPromptSegment::Empty,
            ),
            lexer: LexerState::new(),
        }
    }
}

impl Shell {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read_line(&mut self) -> io::Result<RunState> {
        // choose a prompt
        let prompt = &self.normal_prompt;

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

        // if no input is found, close all indent blocks or return
        if text.trim().len() == 0 {
            match self.lexer.indent_depth() {
                0 => return Ok(RunState::Parsed),
                _ => self.lexer.close_blocks(),
            }
        }

        // parse the text
        let spanned = SpannedLexer::new(self.lexer.lex(&text));
        match grammar::StatementParser::new().parse(&text, spanned) {
            Err(error) => eprintln!("{error}"),
            Ok(statement) => match statement.eval(&mut self.scope) {
                Err(error) => eprintln!("{error}"),
                Ok(value) => match value {
                    Value::None => (), // do nothing
                    _ => println!("{value}"),
                },
            },
        }

        // return successful parse
        Ok(RunState::Parsed)
    }
}
