use serde_json;
use std::path::PathBuf;
use std::str::FromStr;
mod git;
use crate::git::history::GitHistory;
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
    },
}
fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::History { filepath } => {
            let history = GitHistory::parse(&filepath)?;
            eprintln!("{}", serde_json::to_string(&history)?);
        }
    }

    Ok(())
}
