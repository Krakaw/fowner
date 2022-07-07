mod db;
mod errors;
mod git;
mod server;
mod test;

extern crate core;
extern crate log;

use crate::db::models::file::File;
use crate::db::models::project::Project;
use crate::db::processor::Processor;
use crate::db::{Connection, Db};
use crate::errors::FownerError;
use crate::git::manager::GitManager;
use clap::{Parser, Subcommand};
use env_logger::Env;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// Database path
    #[clap(short, long, default_value = "./.data.sqlite3")]
    database_path: PathBuf,

    /// Sub-Commands
    #[clap(subcommand)]
    command: Commands,
}
#[derive(Subcommand, Debug)]
enum Commands {
    /// Serve the HTTP REST API [default: 0.0.0.0:8080]
    Serve {
        /// Listen address
        #[clap(short, long, default_value = "0.0.0.0:8080")]
        listen: SocketAddr,
        /// Monitored repository storage path
        #[clap(short, long, default_value = "./sources")]
        storage_path: PathBuf,
        /// Public asset path
        #[clap(short, long, default_value = "./public")]
        public_asset_path: PathBuf,
    },
    /// Process the git history for a repository
    History {
        /// Path of repository to extract history from
        #[clap(short, long)]
        repo_path: PathBuf,
        /// Git repo url
        #[clap(short = 'u', long)]
        repo_url: Option<String>,
        /// Do not save history in DB
        #[clap(short, long)]
        bypass_save: bool,
        /// Maximum commit hash to extract up to but NOT including
        #[clap(short, long)]
        stop_at_sha: Option<String>,
        /// Fetch github labels
        #[clap(short, long)]
        fetch_github_labels: bool,
    },
    /// Generate a dotfile in the target repo containing all files and their features
    Dotfile {
        /// Path of repository to extract history from
        #[clap(short, long)]
        repo_path: PathBuf,
        /// Dotfile filename
        #[clap(short, long, default_value = ".fowner.features")]
        dotfile: String,
    },
}

#[actix_web::main]
async fn main() -> Result<(), FownerError> {
    env_logger::Builder::from_env(Env::default().default_filter_or("fowner=info")).init();

    let cli = Cli::parse();
    let db = Db::new(&cli.database_path)?;
    // Init runs the migrations on every run
    db.init()?;
    let mut conn = db.pool.get()?;
    let tx = conn.transaction()?;
    let conn = Connection::from(tx);
    match &cli.command {
        Commands::History {
            repo_path,
            repo_url,
            bypass_save,
            stop_at_sha,
            fetch_github_labels,
        } => {
            let git_manager = GitManager {
                path: repo_path.clone(),
                url: repo_url.clone(),
            };
            let processor = Processor::new(git_manager, &conn)?;
            // Fetch the commits from the local repository and insert the required records
            // Projects, Owners, Files, Commits, File Owners
            if *bypass_save {
                eprintln!(
                    "{}",
                    serde_json::to_string(&processor.git_manager.parse_history(None)?)?
                );
            } else {
                let _ = processor
                    .fetch_commits_and_update_db(stop_at_sha.clone(), *fetch_github_labels)
                    .await?;
            }
        }
        Commands::Dotfile { repo_path, dotfile } => {
            let project = Project::load_by_path(repo_path, &conn)?;
            let dotfile_path = repo_path.join(dotfile);
            let path = File::generate_feature_file(project.id, dotfile_path, &conn)?;

            eprintln!("dotfile path = {}", path.canonicalize()?.to_string_lossy());
        }
        Commands::Serve {
            listen,
            storage_path,
            public_asset_path,
        } => {
            server::api::Api::start(db, listen, public_asset_path.clone(), storage_path.clone())
                .await?
        }
    }
    conn.transaction()?.commit()?;

    Ok(())
}
