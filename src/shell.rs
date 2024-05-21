use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

use crate::parser::TokenLines;

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

            let line = match TokenLines::new(buffer.text()).next() {
                Some(line) => line,
                None => continue,
            };

            for result in line {
                match result {
                    Ok((token, span)) => {
                        let str = &buffer.text()[span];
                        println!("'{str}' => {token:?}");
                    }
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
}
