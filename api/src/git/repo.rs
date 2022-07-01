use crate::errors::FownerError;
use crate::git::history::GitHistory;
use crate::git::manager::GitManager;
use chrono::NaiveDateTime;

use std::path::PathBuf;

pub struct GitRepo {
    pub path: PathBuf,
    pub name: Option<String>,
    pub url: Option<String>,
}

impl GitRepo {
    /// Parse the git log output and return GitHistory
    /// The history is chronological ASC
    /// If `since` is passed in it only takes commits 1 second AFTER that datetime
    pub fn parse_history(
        &self,
        since: Option<NaiveDateTime>,
    ) -> Result<Vec<GitHistory>, FownerError> {
        let git_manager = GitManager {
            local_path: self.path.clone(),
            url: None,
        };
        let history_string = git_manager.history(since)?;
        GitHistory::process_rows(history_string)
    }
}
