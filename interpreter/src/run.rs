use std::{fs, path::PathBuf};

use boba_script::lexer::{LexResult, LexerState, LineLexer, TextLines};

pub fn file(path: PathBuf) {
    let name = path.to_string_lossy();
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("Failed to open {name}: {err}");
            return;
        }
    };

    let mut state = LexerState::new();
    let lines = TextLines::new(&text);
    for line in lines {
        let mut lexer = LineLexer::new_with(line, state);
        loop {
            match lexer.generate() {
                LexResult::Token(token) => println!("{token:?}"),
                LexResult::Error(error) => eprintln!("{error}"),
                _ => break,
            }
        }

        state = lexer.consume();
    }
}
