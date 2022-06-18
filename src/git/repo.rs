use crate::db::models::project::{NewProject, Project};
use crate::git::history::GitHistory;
use crate::Db;
use anyhow::Result;
use std::path::{PathBuf, Prefix};
use std::process::Command;
use std::str::FromStr;

pub struct GitRepo {
    pub path: PathBuf,
    pub name: Option<String>,
    pub url: Option<String>,
}

enum GitState {
    Handle,
    Hash,
    Timestamp,
    Summary,
    Files,
}

impl GitRepo {
    pub fn parse(&self, since: usize) -> Result<Vec<GitHistory>> {
        let mut history = vec![];
        let result = Command::new("git")
            .current_dir(&self.path)
            .arg("log")
            .arg("--name-only")
            .arg("--pretty=format:%an%n%h%n%ad%n%s")
            .arg("--date=unix")
            .arg(format!(
                "{}",
                if since > 0 {
                    format!("--since={}", since)
                } else {
                    "".to_string()
                }
            ))
            .arg(".")
            .output()?;

        let s = String::from_utf8(result.stdout)?;
        let mut row = GitHistory {
            handle: "".to_string(),
            hash: "".to_string(),
            timestamp: 0,
            summary: "".to_string(),
            files: vec![],
        };
        let mut state = GitState::Handle;

        for i in s.split('\n') {
            match state {
                GitState::Handle => {
                    row = GitHistory {
                        handle: i.to_string(),
                        hash: "".to_string(),
                        timestamp: 0,
                        summary: "".to_string(),
                        files: vec![],
                    };
                    state = GitState::Hash;
                }
                GitState::Hash => {
                    row.hash = i.to_string();
                    state = GitState::Timestamp;
                }
                GitState::Timestamp => {
                    row.timestamp = usize::from_str(i)?;
                    state = GitState::Summary;
                }
                GitState::Summary => {
                    row.summary = i.to_string();
                    state = GitState::Files;
                }
                GitState::Files => {
                    if i == "" {
                        state = GitState::Handle;
                        history.push(row.clone());
                    } else {
                        row.files.push(i.to_string());
                    }
                }
            }
        }
        Ok(history)
    }

    pub fn store_data(&self, db: &Db) -> Result<(Project)> {
        // Create a new project
        let project = NewProject::from(self).new(db)?;
        // Fetch the commits and store them in the db
        let history = db.store_history(self, None)?;
        Ok((project))
    }
}

impl From<&GitRepo> for NewProject {
    fn from(repo: &GitRepo) -> Self {
        NewProject {
            name: repo.name.clone(),
            repo_url: repo.url.clone(),
            path: repo.path.clone(),
        }
    }
}
