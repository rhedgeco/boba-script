use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

use crate::{ast::Expr, parser::TokenSource};

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
                    _ => buffer,
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
            let mut tokens = TokenSource::new(&buffer);

            // parse expression
            match Expr::parse_atom(&mut tokens) {
                Ok(expr) => println!("{expr:?}"),
                Err(error) => eprintln!("{error}"),
            }
        }
    }
}
