use std::{fs, path::PathBuf};

use boba_script::{
    engine::{scope::LocalScope, Eval},
    lexer::LexerState,
    parser::{grammar, spanned::SpannedLexer},
};

pub fn file(path: PathBuf) {
    // load file
    let name = path.to_string_lossy();
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("Failed to open {name}: {err}");
            return;
        }
    };

    // skip empty input
    if text.trim().len() == 0 {
        println!("file is empty");
        return;
    }

    // parse the text
    let mut state = LexerState::new();
    let lexer = SpannedLexer::new(state.lex(&text));
    let statements = match grammar::StatementListParser::new().parse(&text, lexer) {
        Ok(statements) => statements,
        Err(error) => {
            eprintln!("{error}");
            return;
        }
    };

    // execute the file
    let mut scope = LocalScope::new();
    for statement in statements {
        match statement.eval(&mut scope) {
            Ok(value) => println!("{value}"),
            Err(error) => {
                eprintln!("{error}");
                return;
            }
        }
    }
}
