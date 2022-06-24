use crate::git::history::GitHistory;
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use regex::Regex;
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;

pub struct GitRepo {
    pub path: PathBuf,
    pub name: Option<String>,
    pub url: Option<String>,
}

#[derive(PartialEq)]
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
            "--pretty=format:---%n%an%n%h%n%ad%n%s".to_string(),
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
        let mut row = GitHistory::default();
        let mut state = GitState::Handle;
        // Extracts features from an appended [Feature,Feature] list in the commit message
        let re = Regex::new(r"\[([\w,]+)\]$")?;

        for line in s.split('\n') {
            let line = line.to_string();
            if line == "---" {
                // This pattern denotes the start of a new record
                if state == GitState::Files {
                    history.push(row.clone());
                    state = GitState::Handle;
                }
                continue;
            }

            match state {
                GitState::Handle => {
                    row = GitHistory {
                        handle: line,
                        ..GitHistory::default()
                    };
                    state = GitState::Hash;
                }
                GitState::Hash => {
                    row.hash = line;
                    state = GitState::Timestamp;
                }
                GitState::Timestamp => {
                    row.timestamp = usize::from_str(&line)?;
                    state = GitState::Summary;
                }
                GitState::Summary => {
                    row.summary = line.clone();
                    if let Some(captures) = re.captures(&line) {
                        let features = captures
                            .get(1)
                            .map(|r| r.as_str().split(",").collect())
                            .unwrap_or_else(|| vec![])
                            .iter()
                            .map(|s| s.to_string())
                            .collect::<Vec<String>>();
                        row.features = features;
                    }
                    state = GitState::Files;
                }
                GitState::Files => {
                    if line.is_empty() {
                        state = GitState::Handle;
                        history.push(row.clone());
                    } else {
                        row.files.push(line.to_string());
                    }
                }
            }
        }
        Ok(history)
    }
}
