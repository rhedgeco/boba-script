use std::ops::Deref;

use ariadne::Source;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

use crate::{ast::Statement, parser::BufferSource, Engine};

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
        let mut engine = Engine::new();
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
                Err(error) => {
                    error
                        .as_ariadne("shell")
                        .eprint(("shell", buffer.clone()))
                        .unwrap();
                }
                Ok(statement) => match statement {
                    Statement::Expr(expr) => {
                        match engine.eval(&expr) {
                            Ok(value) => println!("{value}"),
                            Err(error) => {
                                error
                                    .as_ariadne("shell")
                                    .eprint(("shell", buffer.clone()))
                                    .unwrap();
                                continue;
                            }
                        };
                    }
                    Statement::Assign(assign) => {
                        // evaluate expression
                        let value = match engine.eval(&assign.expr) {
                            Ok(value) => value,
                            Err(error) => {
                                error
                                    .as_ariadne("shell")
                                    .eprint(("shell", buffer.clone()))
                                    .unwrap();
                                continue;
                            }
                        };

                        // assign variable
                        let ident = assign.ident.deref();
                        println!("{ident} = {value}");
                        engine.push_var(ident.clone(), value);
                    }
                },
            }
        }
    }
}
