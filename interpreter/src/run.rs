use std::{fs, path::PathBuf};

use boba_script::lexer::{BobaCache, Lexer};

pub fn file(path: PathBuf) {
    let name = path.to_string_lossy();
    let mut cache = BobaCache::new();
    let data = match fs::read_to_string(&path) {
        Ok(data) => cache.store(name, data),
        Err(err) => {
            eprintln!("Failed to open {name}: {err}");
            return;
        }
    };

    let lexer = Lexer::new(data);
    for result in lexer {
        match result {
            Ok(token) => println!("{token}"),
            Err(error) => eprintln!("{error}"),
        }
    }
}
