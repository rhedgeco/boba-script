use std::path::PathBuf;

use bobarista::builder;
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
    Build {
        #[arg(short, long)]
        workdir: Option<PathBuf>,
    },
}

fn main() {
    let cli = BobaCli::parse();
    match cli.command {
        Commands::Build { workdir } => builder::build_project(workdir),
    }
}
