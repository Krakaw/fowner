use serde_json;
use std::path::PathBuf;
mod db;
mod git;

use crate::db::Db;
use crate::git::repo::GitRepo;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Database path
    #[clap(short, long, default_value = "./.data.sqlite3")]
    database_path: PathBuf,
    #[clap(subcommand)]
    command: Commands,
}
#[derive(Subcommand, Debug)]
enum Commands {
    History {
        /// Path of repository to extract history from
        #[clap(short, long)]
        filepath: PathBuf,
        /// Project Name
        #[clap(short, long)]
        name: Option<String>,
        /// Git repo url
        #[clap(short, long)]
        repo_url: Option<String>,
        /// Fetch the latest since
        #[clap(short, long)]
        since: Option<i64>,
    },
    Migrate,
}
fn main() -> Result<()> {
    let cli = Cli::parse();
    let db = Db::new(&cli.database_path)?;

    match &cli.command {
        Commands::History {
            filepath,
            since,
            name,
            repo_url,
        } => {
            let repo = GitRepo {
                path: filepath.clone(),
                name: name.clone(),
                url: repo_url.clone(),
            };
            let result = repo.store_data(&db)?;

            // let history = db.store_history(repo, since.clone())?;

            // eprintln!("{}", serde_json::to_string(&history)?);
        }
        Commands::Migrate => db.init()?,
    }

    Ok(())
}
