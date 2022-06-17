use serde_json;
use std::path::PathBuf;
mod git;
use crate::git::history::GitRepo;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}
#[derive(Subcommand, Debug)]
enum Commands {
    History {
        /// Path of repository to extract history from
        #[clap(short, long)]
        filepath: PathBuf,
        /// Fetch the latest since
        #[clap(short, long, default_value = "0")]
        since: usize,
    },
}
fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::History { filepath, since } => {
            let repo = GitRepo(filepath.clone());
            let history = repo.parse(Some(*since))?;
            eprintln!("{}", serde_json::to_string(&history)?);
        }
    }

    Ok(())
}
