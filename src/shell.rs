use std::ops::Deref;

use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

use crate::{
    parser::{ast::Statement, TokenLines},
    Engine,
};

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
        let mut engine = Engine::new();
        let mut shell = Session::new();
        loop {
            let buffer = match shell.line_editor.read_line(&shell.prompt) {
                Ok(Signal::Success(buffer)) => match buffer.trim().len() {
                    0 => continue,
                    _ => ariadne::Source::from(buffer),
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

            let (_indent, mut line) = match TokenLines::new(buffer.text()).next() {
                Some(line_data) => line_data,
                None => continue,
            };

            match Statement::parse(&mut line) {
                Ok(statement) => match statement.deref() {
                    Statement::LetAssign(var, expr) => {
                        let value = match engine.eval(&expr) {
                            Ok(value) => value,
                            Err(error) => {
                                error
                                    .to_ariadne("shell")
                                    .eprint(("shell", buffer.clone()))
                                    .unwrap();
                                continue;
                            }
                        };

                        engine.set_var(var.deref(), value);
                    }
                    Statement::Assign(var, expr) => {
                        let new_value = match engine.eval(&expr) {
                            Ok(value) => value,
                            Err(error) => {
                                error
                                    .to_ariadne("shell")
                                    .eprint(("shell", buffer.clone()))
                                    .unwrap();
                                continue;
                            }
                        };

                        match engine.get_var_mut(var) {
                            Ok(old_value) => *old_value = new_value,
                            Err(error) => {
                                error
                                    .to_ariadne("shell")
                                    .eprint(("shell", buffer.clone()))
                                    .unwrap();
                                continue;
                            }
                        }
                    }
                    Statement::Expr(expr) => match engine.eval(&expr) {
                        Ok(value) => println!("{value}"),
                        Err(error) => {
                            error
                                .to_ariadne("shell")
                                .eprint(("shell", buffer.clone()))
                                .unwrap();
                            continue;
                        }
                    },
                },
                Err(error) => {
                    error
                        .to_ariadne("shell")
                        .eprint(("shell", buffer.clone()))
                        .unwrap();
                    continue;
                }
            }
        }
    }
}
