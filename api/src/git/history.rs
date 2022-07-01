use crate::errors::FownerError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GitHistory {
    pub handle: String,
    pub sha: String,
    pub parent_sha: Option<String>,
    pub timestamp: usize,
    pub summary: String,
    pub files: Vec<String>,
    pub features: Vec<String>,
}

#[derive(PartialEq)]
enum GitState {
    Handle,
    Sha,
    ParentSha,
    Timestamp,
    Summary,
    Files,
}

impl GitHistory {
    /// Processes the specifically formatted git log to generate a vec of `GitHistory`
    pub fn parse(history_string: String) -> Result<Vec<GitHistory>, FownerError> {
        let mut history = vec![];
        let mut row = GitHistory::default();
        let mut state = GitState::Handle;
        // Extracts features from an appended [Feature,Feature] list in the commit message
        let re = Regex::new(r"\[([\w ,-]+)\]$")?;

        for line in history_string.split('\n') {
            let line = line.trim().to_string();
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
                    state = GitState::Sha;
                }
                GitState::Sha => {
                    row.sha = line;
                    state = GitState::ParentSha;
                }
                GitState::ParentSha => {
                    row.parent_sha = match line.trim() {
                        "" => None,
                        _ => Some(
                            // Always use the last commit as the parent
                            line.trim()
                                .split(' ')
                                .last()
                                .unwrap_or_default()
                                .to_string(),
                        ),
                    };
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
                            .map(|r| r.as_str().split(',').collect())
                            .unwrap_or_else(Vec::new)
                            .iter()
                            .map(|s| s.trim().to_string())
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
        history.reverse();
        Ok(history)
    }
}

#[cfg(test)]
mod test {
    use crate::git::history::GitHistory;

    #[test]
    fn process_rows() {
        let history_string = r#"---
Krakaw
74ebe78fe948f83d42a59d021b5a411b7ac13981
685dc3d64c54719dadd46b9b7bc4cb0a994728e1 43ce09e6475f4013fc00a6bdfefc4a53e4ddff68
1655513699
Merge branch 'main' of github.com:Krakaw/fowner
---
Krakaw
685dc3d64c54719dadd46b9b7bc4cb0a994728e1
790f3db24ba26480725dbfc52594b4ce5d4a0b13
1655513635
Added models and initial DB interactions
.gitignore
Cargo.toml
src/db/migrations.rs
src/db/mod.rs
src/db/models/commit.rs
src/db/models/feature.rs
src/db/models/file.rs
src/db/models/mod.rs
src/db/models/owner.rs
src/db/models/project.rs
src/git/history.rs
src/git/mod.rs
src/git/repo.rs
src/main.rs

---
Krakaw
43ce09e6475f4013fc00a6bdfefc4a53e4ddff68
790f3db24ba26480725dbfc52594b4ce5d4a0b13
1655451712
Initial commit of feature tracking based on git history
src/git/history.rs
src/main.rs

---
Krakaw
790f3db24ba26480725dbfc52594b4ce5d4a0b13
c60c24663d3b67fdee8079a18cbe40c843932b48
1655395513
Initial commit of feature tracking based on git history [Core_Feature-1, History 2]
.gitignore
Cargo.toml
README.md
src/git/history.rs
src/git/mod.rs
src/main.rs

---
Keith Simon
c60c24663d3b67fdee8079a18cbe40c843932b48

1655391971
Initial commit [Core_Feature-1]
.gitignore
README.md

"#
        .to_string();
        let history = GitHistory::parse(history_string).unwrap();
        assert_eq!(history.len(), 5);
        let initial = history.first().unwrap();
        assert_eq!(initial.sha, "c60c24663d3b67fdee8079a18cbe40c843932b48");
        assert_eq!(initial.handle, "Keith Simon");
        assert_eq!(initial.features, vec!["Core_Feature-1"]);
        assert_eq!(initial.files, vec![".gitignore", "README.md"]);
        assert_eq!(initial.parent_sha, None);

        let multiple_features = history.get(1).unwrap();
        assert_eq!(
            multiple_features.features,
            vec!["Core_Feature-1", "History 2"]
        );

        let last = history.last().unwrap();
        assert_eq!(last.sha, "74ebe78fe948f83d42a59d021b5a411b7ac13981");
        assert_eq!(
            last.parent_sha,
            Some("43ce09e6475f4013fc00a6bdfefc4a53e4ddff68".to_string())
        );
    }
}
