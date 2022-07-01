use crate::FownerError;
use chrono::{Duration, NaiveDateTime};
use log::{debug, trace};
use std::path::PathBuf;
use std::process::Command;

const GIT_HISTORY_LOG_FORMAT: &str = "---%n%an%n%H%n%P%n%ad%n%s";

pub struct GitManager {
    pub local_path: PathBuf,
    pub url: Option<String>,
}

impl GitManager {
    pub fn init(local_path: PathBuf, url: Option<String>) -> Result<Self, FownerError> {
        if url.is_none() {
            return Err(FownerError::Internal(
                "repo_url cannot be blank to init GitManager".to_string(),
            ));
        }
        let git_manager = Self { local_path, url };

        if !git_manager.is_valid_repo() {
            // This path has not been instantiated, attempt to clone the repo
            git_manager.clone()?;
        }
        Ok(git_manager)
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
            .current_dir(&self.local_path)
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
            .current_dir(&self.local_path)
            .arg("fetch")
            .output()
            .map_err(|e| FownerError::GitError(format!("Fetch error {}", e)))?;
        Ok(())
    }

    pub fn is_valid_repo(&self) -> bool {
        Command::new("git")
            .current_dir(&self.local_path)
            .arg("status")
            .output()
            .is_ok()
    }

    pub fn clone(&self) -> Result<(), FownerError> {
        Command::new("git")
            .current_dir(&self.local_path.parent().ok_or_else(|| {
                FownerError::NotFound(format!("Invalid local path {:?}", self.local_path))
            })?)
            .arg("clone")
            .arg("--no-checkout")
            .arg(&self.url.clone().unwrap_or_default())
            .arg(&self.local_path)
            .output()
            .map_err(|e| FownerError::GitError(format!("Clone error {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::git::manager::GitManager;
    use crate::test::tests::TestHandler;

    #[test]
    fn initialize_new_repo() {
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

        // Fetch should succeed
        assert!(git_manager.fetch().is_ok());
    }
}
