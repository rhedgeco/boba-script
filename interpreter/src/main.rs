use std::path::PathBuf;

use boba::{run, shell::RunState, Shell};
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
        Some(path) => run::file(path),
        None => {
            let mut shell = Shell::new();
            loop {
                match shell.read_statement() {
                    Err(error) => panic!("{error}"),
                    Ok(RunState::Success) => continue,
                    Ok(RunState::CtrlC) => {
                        println!("Aborting...");
                        break;
                    }
                    Ok(RunState::CtrlD) => {
                        println!("Aborting...");
                        break;
                    }
                }
            }
        }
    }
}
