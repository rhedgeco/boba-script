use std::{fs, path::PathBuf};

use boba_script::lexer::LexerState;

pub fn file(path: PathBuf) {
    let name = path.to_string_lossy();
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("Failed to open {name}: {err}");
            return;
        }
    };

    for result in LexerState::new().lex(&text).filtered() {
        match result {
            Ok(token) => println!("{token}"),
            Err(error) => eprintln!("{error}"),
        }
    }
}
