use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

pub struct GitRepo(pub PathBuf);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GitHistory {
    pub handle: String,
    pub hash: String,
    pub timestamp: usize,
    pub summary: String,
    pub files: Vec<String>,
}

enum GitState {
    Handle,
    Hash,
    Timestamp,
    Summary,
    Files,
}

impl GitRepo {
    pub fn parse(&self, since: Option<usize>) -> Result<Vec<GitHistory>> {
        let mut history = vec![];
        let mut result = Command::new("git")
            .current_dir(&self.0)
            .arg("log")
            .arg("--name-only")
            .arg("--pretty=format:%an%n%h%n%ad%n%s")
            .arg("--date=unix")
            .arg(".");
        if let Some(since) = since {
            result = result.arg(&format!("--since={}", since));
        }
        let result = result.output()?;
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
}
