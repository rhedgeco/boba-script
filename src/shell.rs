use std::env;

use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

use crate::{
    engine::Value,
    parser::{ast::Statement, lexer::Lexer},
    BobaCache, Engine,
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
        // create a cache for all items in the current directory
        let mut cache = match env::current_dir() {
            Ok(path) => BobaCache::new(path),
            Err(e) => {
                eprintln!("{e}");
                std::process::exit(-1);
            }
        };

        let mut shell = Session::new();
        let mut engine = Engine::new();
        loop {
            let data = match shell.line_editor.read_line(&shell.prompt) {
                Ok(Signal::Success(buffer)) => cache.store("shell", buffer),
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

            let mut tokens = match Lexer::new(data).next() {
                None => continue,
                Some(Ok(tokens)) => tokens,
                Some(Err(e)) => {
                    e.report().eprint(&mut cache).unwrap();
                    continue;
                }
            };

            match Statement::parse(&mut tokens) {
                Ok(statement) => match engine.eval_statement(&statement) {
                    Ok(Value::None) => continue,
                    Ok(value) => println!("{value}"),
                    Err(e) => {
                        e.report().eprint(&mut cache).unwrap();
                        continue;
                    }
                },
                Err(e) => {
                    e.report().eprint(&mut cache).unwrap();
                    continue;
                }
            }
        }
    }
}
