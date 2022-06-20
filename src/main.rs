use std::path::PathBuf;
mod controllers;
mod db;
mod errors;
mod git;

extern crate log;
use crate::db::models::file::File;
use crate::db::models::project::Project;
use crate::db::processor::Processor;
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
    },
    FileOwners {
        /// File name
        #[clap(short, long)]
        name: String,
        /// Path of repository to extract history from
        #[clap(short, long)]
        filepath: PathBuf,
    },
    Serve,
    Migrate,
}

#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    let cli = Cli::parse();
    let db = Db::new(&cli.database_path)?;

    match &cli.command {
        Commands::History {
            filepath,
            name,
            repo_url,
        } => {
            let repo = GitRepo {
                path: filepath.clone(),
                name: name.clone(),
                url: repo_url.clone(),
            };
            let mut processor = Processor::new(repo, &db)?;
            // Fetch the commits from the local repository and insert the required records
            // Projects, Owners, Files, Commits, File Owners
            let _ = processor.fetch_commits_and_update_db()?;
            // TODO: Come up with a solution for matching features with files (I'm thinking a dotfile in the repo)

            // let history = db.store_history(repo, since.clone())?;

            // eprintln!("{}", serde_json::to_string(&history)?);
        }
        Commands::FileOwners { filepath, name } => {
            let project = Project::load_by_path(filepath, &db)?;

            let file = File::load_by_path(project.id, name.clone(), &db)?;
            let owners = file.get_owners(&db)?;
            eprintln!("{}", serde_json::to_string(&owners)?);
        }
        Commands::Migrate => db.init()?,
        Commands::Serve => controllers::Server::start(db).await?,
    }

    Ok(())
}
