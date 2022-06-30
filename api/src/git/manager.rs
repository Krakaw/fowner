use crate::{FownerError, Project};
use actix_web::http::Uri;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use std::str::FromStr;

pub struct GitManager {
    local_path: PathBuf,
    repo_url: String,
}

impl GitManager {
    pub fn init(local_path: PathBuf, repo_url: String) -> Result<Self, FownerError> {
        let git_manager = Self {
            local_path,
            repo_url,
        };

        if !git_manager.is_valid_repo() {
            // This path has not been instantiated, attempt to clone the repo
            git_manager.clone()?;
        }
        Ok(git_manager)
    }

    pub fn fetch(&self) -> Result<(), FownerError> {
        Command::new("git")
            .current_dir(&self.local_path)
            .arg("fetch")
            .status()
            .map_err(|e| FownerError::GitError(format!("Fetch error {}", e)))?;
        Ok(())
    }

    pub fn is_valid_repo(&self) -> bool {
        Command::new("git")
            .current_dir(&self.local_path)
            .arg("status")
            .status()
            .is_ok()
    }

    pub fn clone(&self) -> Result<(), FownerError> {
        Command::new("git")
            .current_dir(
                &self
                    .local_path
                    .parent()
                    .ok_or(FownerError::NotFound(format!(
                        "Invalid local path {:?}",
                        self.local_path
                    )))?,
            )
            .arg("clone")
            .arg("--no-checkout")
            .arg(&self.repo_url)
            .arg(&self.local_path)
            .status()
            .map_err(|e| FownerError::GitError(format!("Clone error {}", e)))?;
        Ok(())
    }
}
