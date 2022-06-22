use crate::git::history::GitHistory;
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use std::path::PathBuf;
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
    pub fn parse(&self, since: Option<NaiveDateTime>) -> Result<Vec<GitHistory>> {
        let mut history = vec![];
        let mut args = vec![
            "--no-pager".to_string(),
            "log".to_string(),
            "--name-only".to_string(),
            "--pretty=format:%an%n%h%n%ad%n%s".to_string(),
            "--date=unix".to_string(),
        ];

        if let Some(since) = since {
            let after = format!("--after={}", since);
            args.push(after);
        }
        let result = Command::new("git")
            .current_dir(&self.path)
            .args(args)
            .arg(".")
            .output()?;
        if !result.status.success() {
            return Err(anyhow!(String::from_utf8(result.stderr)?));
        }
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
                    if i.is_empty() {
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
