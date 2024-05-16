use ariadne::Source;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

use crate::{ast::Statement, parser::BufferSource};

pub struct Session {
    prompt: DefaultPrompt,
    line_editor: Reedline,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            prompt: DefaultPrompt::new(
                DefaultPromptSegment::Basic(format!("boba ")),
                DefaultPromptSegment::Empty,
            ),
            line_editor: Reedline::create(),
        }
    }
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_console() {
        let mut shell = Session::new();
        loop {
            let buffer = match shell.line_editor.read_line(&shell.prompt) {
                Ok(Signal::Success(buffer)) => match buffer.len() {
                    0 => continue,
                    _ => Source::from(buffer),
                },
                Ok(Signal::CtrlD) => {
                    println!("Closing Shell...");
                    return;
                }
                Ok(Signal::CtrlC) => {
                    println!("Aborting...");
                    return;
                }
                Err(e) => {
                    eprintln!("Input Error: {e}");
                    continue;
                }
            };

            // create token source
            let mut source = BufferSource::new(buffer.text());

            // parse expression
            match Statement::parse(&mut source) {
                Ok(statement) => println!("{statement:?}"),
                Err(report) => {
                    for error in report.errors() {
                        error
                            .as_ariadne("shell")
                            .eprint(("shell", buffer.clone()))
                            .unwrap();
                    }
                }
            }
        }
    }
}
