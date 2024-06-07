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

    pub fn start_console(&mut self) {
        let mut engine = Engine::new();
        let mut cache = BobaCache::new();
        loop {
            let data = match self.line_editor.read_line(&self.prompt) {
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

            let mut lexer = Lexer::new(data);
            if lexer.peek().is_none() {
                continue; // if there are no tokens, do nothing
            }

            match Statement::parse(&mut lexer) {
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
