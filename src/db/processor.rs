use crate::db::models::commit::{Commit, NewCommit};
use crate::db::models::file::{File, NewFile};
use crate::db::models::file_owner::NewFileOwner;
use crate::db::models::owner::NewOwner;
use crate::db::models::project::{NewProject, Project};
use crate::{Db, GitRepo};
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use log::debug;

pub struct Processor<'a> {
    pub db: &'a Db,
    pub repo: GitRepo,
    pub project: Option<Project>,
}

impl<'a> Processor<'a> {
    pub fn new(repo: GitRepo, db: &'a Db) -> Result<Self> {
        Ok(Processor {
            db,
            repo,
            project: None,
        })
    }

    pub fn fetch_commits_and_update_db(&mut self) -> Result<()> {
        let _project = self.create_or_load_project()?;
        debug!("Created project {:?}", _project);
        let _ = self.fetch_history_and_store_data()?;
        Ok(())
    }
    pub fn create_or_load_project(&mut self) -> Result<Project> {
        let project = NewProject::from(&self.repo).new(self.db)?;
        self.project = Some(project.clone());
        Ok(project)
    }

    pub fn fetch_history_and_store_data(&mut self) -> Result<()> {
        let latest_commit = self.get_most_recent_commit();
        debug!("Latest Commit: {}", latest_commit);
        let history = self.repo.parse((latest_commit + 1) as usize)?;
        let project = self.project.clone().ok_or(anyhow!("Missing project"))?;
        for git_history in history.clone() {
            eprintln!("git_history = {:?}", git_history);
            // For each GitHistory
            // 1. We need to create an Owner from the handle
            let owner = NewOwner {
                handle: git_history.handle,
                name: None,
            }
            .new(self.db)?;
            // 2a. We need to extract all of the files and create a new File entry for each that is linked to the project
            for file_path in git_history.files {
                let file = NewFile {
                    project_id: project.id,
                    path: file_path,
                }
                .new(self.db)?;
                // 2b. We need to create a Commit for the hash for each file entry
                let commit_date = NaiveDateTime::from_timestamp(git_history.timestamp as i64, 0);
                NewCommit {
                    file_id: file.id,
                    sha: git_history.hash.clone(),
                    description: git_history.summary.clone(),
                    commit_time: commit_date.clone(),
                }
                .new(self.db)?;
                // 2c. We need to create a FileOwner for each file
                NewFileOwner {
                    file_id: file.id,
                    owner_id: owner.id,
                    action_date: commit_date,
                }
                .new(self.db)?;
            }
        }
        Ok(())
    }

    fn get_most_recent_commit(&self) -> i64 {
        Commit::fetch_latest(self.db)
            .map(|c| c.commit_time.timestamp())
            .unwrap_or_else(|_| 0_i64)
    }
}
