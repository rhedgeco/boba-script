use std::{fs, path::Path};

pub fn read_to_source(input: impl AsRef<Path>) -> ariadne::Source {
    let input = input.as_ref();
    match fs::read_to_string(input) {
        Ok(source) => ariadne::Source::from(source),
        Err(e) => {
            eprintln!("Failed to read {}: {e}", input.display());
            std::process::exit(-1);
        }
    }
}
