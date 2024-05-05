use boba::shell;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct BobaCli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Shell,
}

fn main() {
    let cli = BobaCli::parse();
    match cli.command {
        Commands::Shell => shell::start_session(),
    }
}
