use crate::git::history::GitHistory;
use crate::FownerError;
use chrono::{Duration, NaiveDateTime};
use log::{debug, trace};
use std::path::PathBuf;
use std::process::Command;

const GIT_HISTORY_LOG_FORMAT: &str = "---%n%an%n%H%n%P%n%ad%n%s";

pub struct GitManager {
    pub path: PathBuf,
    pub url: Option<String>,
}

impl GitManager {
    pub fn init(path: PathBuf, url: Option<String>) -> Result<Self, FownerError> {
        let git_manager = Self { path, url };
        if !git_manager.path.exists() {
            // Create the path if it doesn't exist
            std::fs::create_dir_all(&git_manager.path)?;
        }
        if !git_manager.is_valid_repo()? {
            // This path has not been instantiated, attempt to clone the repo
            git_manager.clone()?;
        }
        Ok(git_manager)
    }

    /// Parse the git log output and return GitHistory
    /// The history is chronological ASC
    /// If `since` is passed in it only takes commits 1 second AFTER that datetime
    pub fn parse_history(
        &self,
        since: Option<NaiveDateTime>,
    ) -> Result<Vec<GitHistory>, FownerError> {
        GitHistory::parse(self.history(since)?)
    }

    /// Returns the raw git log history string
    pub fn history(&self, since: Option<NaiveDateTime>) -> Result<String, FownerError> {
        let mut args = vec![
            "--no-pager".to_string(),
            "log".to_string(),
            "--name-only".to_string(),
            format!("--pretty=format:{}", GIT_HISTORY_LOG_FORMAT),
            "--date=unix".to_string(),
        ];

        if let Some(since) = since {
            let after = format!(
                "--after=\"{}\"",
                since
                    .checked_add_signed(Duration::seconds(1))
                    .unwrap_or(since)
                    .format("%Y-%m-%dT%H:%M:%S.0Z")
            );
            debug!("Fetching Commits After: {}", after);
            args.push(after);
        }
        trace!("git {}", args.join(" "));
        let result = Command::new("git")
            .current_dir(&self.path)
            .args(args)
            .arg(".")
            .output()?;
        if !result.status.success() {
            return Err(FownerError::Execution(String::from_utf8(result.stderr)?));
        }
        Ok(String::from_utf8(result.stdout)?)
    }

    pub fn fetch(&self) -> Result<(), FownerError> {
        Command::new("git")
            .current_dir(&self.path)
            .arg("fetch")
            .output()
            .map_err(|e| FownerError::GitError(format!("Fetch error {}", e)))?;
        Ok(())
    }

    pub fn is_valid_repo(&self) -> Result<bool, FownerError> {
        let result = Command::new("git")
            .current_dir(&self.path)
            .arg("status")
            .output()
            .map_err(|e| FownerError::GitError(format!("Status error {}", e)));
        match result {
            Ok(output) => Ok(output.status.success()),
            Err(e) => {
                debug!("Checking if repo is valid failed {:?}", e);
                Ok(false)
            }
        }
    }

    pub fn clone(&self) -> Result<(), FownerError> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .arg("clone")
            .arg("--no-checkout")
            .arg(&self.url.clone().unwrap_or_default())
            .arg(".")
            .output()
            .map_err(|e| {
                eprintln!("e = {:?}", e);
                FownerError::GitError(format!("Clone error {}", e))
            })?;
        debug!(
            "Clone Result {}\n Clone Error: {}",
            String::from_utf8(output.stdout)?,
            String::from_utf8(output.stderr)?
        );
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::git::manager::GitManager;
    use crate::test::tests::TestHandler;

    #[test]
    fn initialize_new_remote_repo() {
        let handler = TestHandler::init();
        let tmp_dir = handler.tmp_dir.clone();
        let repo_dir = tmp_dir.join("fowner");
        let git_manager = GitManager::init(
            repo_dir.clone(),
            Some("https://github.com/Krakaw/empty.git".to_string()),
        )
        .unwrap();
        let history = git_manager.history(None).unwrap();

        assert!(history.starts_with("---"));
        assert!(history.contains("d2b7bc86de36a40c2f32cf44c1931a38163bfb51"));

        // Fetch should succeed
        assert!(git_manager.fetch().is_ok());
    }
}
