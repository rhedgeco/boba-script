use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

use crate::{
    parser::{ast::Expr, TokenLines},
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

            match Expr::parse(&mut line) {
                Ok(expr) => match engine.eval(&expr) {
                    Ok(value) => println!("{value}"),
                    Err(error) => error
                        .to_ariadne("shell")
                        .eprint(("shell", buffer.clone()))
                        .unwrap(),
                },
                Err(error) => {
                    error
                        .to_ariadne("shell")
                        .eprint(("shell", buffer.clone()))
                        .unwrap();
                }
            }
        }
    }
}
