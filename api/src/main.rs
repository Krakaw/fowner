use std::net::SocketAddr;
use std::path::PathBuf;
mod controllers;
mod db;
mod errors;
mod git;
mod test;

extern crate core;
extern crate log;

use crate::db::models::file::File;
use crate::db::models::project::Project;
use crate::db::processor::Processor;
use crate::db::Db;

use crate::errors::FownerError;
use crate::git::manager::GitManager;
use clap::{Parser, Subcommand};
use env_logger::Env;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Database path
    #[clap(short, long, default_value = "./.data.sqlite3")]
    database_path: PathBuf,
    /// Temp repo path
    #[clap(short, long, default_value = "./data")]
    temp_repo_path: PathBuf,
    /// Sub-Commands
    #[clap(subcommand)]
    command: Commands,
}
#[derive(Subcommand, Debug)]
enum Commands {
    /// Process the git history for a repository
    History {
        /// Path of repository to extract history from
        #[clap(short, long)]
        filepath: PathBuf,
        /// Git repo url
        #[clap(short, long)]
        repo_url: Option<String>,
        /// Do not save history in DB
        #[clap(short, long)]
        bypass_save: bool,
    },
    FileOwners {
        /// File name
        #[clap(short, long)]
        name: String,
        /// Path of repository to extract history from
        #[clap(short, long)]
        filepath: PathBuf,
    },
    GenerateDotfile {
        /// Path of repository to extract history from
        #[clap(short, long)]
        filepath: PathBuf,
        /// Dotfile filename
        #[clap(short, long, default_value = ".fowner.features")]
        dotfile: String,
    },
    /// Serve the HTTP REST API [default: 0.0.0.0:8080]
    Serve {
        /// Listen address
        #[clap(short, long, default_value = "0.0.0.0:8080")]
        listen: SocketAddr,
    },
    /// Run the migrations for the database
    Migrate,
}

#[actix_web::main]
async fn main() -> Result<(), FownerError> {
    env_logger::Builder::from_env(Env::default().default_filter_or("fowner=info")).init();

    let cli = Cli::parse();
    let db = Db::new(&cli.database_path)?;
    let temp_repo_path = cli.temp_repo_path;

    match &cli.command {
        Commands::History {
            filepath,
            repo_url,
            bypass_save,
        } => {
            let git_manager = GitManager {
                path: filepath.clone(),
                url: repo_url.clone(),
            };
            let processor = Processor::new(git_manager, &db)?;
            // Fetch the commits from the local repository and insert the required records
            // Projects, Owners, Files, Commits, File Owners
            if *bypass_save {
                eprintln!(
                    "{}",
                    serde_json::to_string(&processor.git_manager.parse_history(None)?)?
                );
            } else {
                let _ = processor.fetch_commits_and_update_db()?;
            }
        }
        Commands::GenerateDotfile { filepath, dotfile } => {
            let project = Project::load_by_path(filepath, &db)?;
            let dotfile_path = filepath.join(dotfile);
            let path = File::generate_feature_file(project.id, dotfile_path, &db)?;

            eprintln!("dotfile path = {}", path.canonicalize()?.to_string_lossy());
        }
        Commands::FileOwners { filepath, name } => {
            let project = Project::load_by_path(filepath, &db)?;

            let file = File::load_by_path(project.id, name.clone(), &db)?;
            let owners = file.get_owners(&db)?;
            eprintln!("{}", serde_json::to_string(&owners)?);
        }
        Commands::Migrate => db.init()?,
        Commands::Serve { listen } => {
            controllers::Server::start(db, listen, temp_repo_path).await?
        }
    }

    Ok(())
}
