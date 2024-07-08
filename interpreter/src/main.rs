use std::path::PathBuf;

use boba::{run, shell};
use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct BobaCli {
    file: Option<PathBuf>,
}

fn main() {
    let cli = BobaCli::parse();
    match cli.file {
        None => shell::session(),
        Some(path) => run::file(path),
    }
}
