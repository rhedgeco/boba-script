use std::{fs, path::PathBuf};

pub fn file(path: PathBuf) {
    let name = path.to_string_lossy();
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("Failed to open {name}: {err}");
            return;
        }
    };

    println!("{text}")
}
