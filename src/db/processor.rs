use crate::db::models::commit::{Commit, NewCommit};
use crate::db::models::file::NewFile;
use crate::db::models::file_owner::NewFileOwner;
use crate::db::models::owner::NewOwner;
use crate::db::models::project::{NewProject, Project};
use crate::{Db, GitRepo};
use anyhow::Result;
use chrono::NaiveDateTime;

pub struct Processor<'a> {
    pub db: &'a Db,
    pub repo: GitRepo,
    pub project: Project,
}

impl<'a> Processor<'a> {
    pub fn new(repo: GitRepo, db: &'a Db) -> Result<Self> {
        let project = NewProject::from(&repo).new(db)?;
        Ok(Processor { db, repo, project })
    }

    pub fn fetch_commits_and_update_db(&mut self) -> Result<()> {
        let _ = self.fetch_history_and_store_data()?;
        Ok(())
    }

    pub fn fetch_history_and_store_data(&mut self) -> Result<()> {
        let latest_commit = self.get_most_recent_commit();
        let history = self.repo.parse(latest_commit)?;
        let project = self.project.clone();
        let project_id = project.id;
        for git_history in history.clone() {
            // For each GitHistory
            // 1. We need to create an Owner from the handle
            let owner = NewOwner {
                handle: git_history.handle,
                name: None,
            }
            .new(self.db)?;
            // 2. We need to create a Commit for the hash
            let commit_date = NaiveDateTime::from_timestamp(git_history.timestamp as i64, 0);
            let sha = git_history.hash.clone();
            NewCommit {
                project_id,
                sha: sha.clone(),
                description: git_history.summary.clone(),
                commit_time: commit_date.clone(),
            }
            .new(self.db)?;
            // 3a. We need to extract all of the files and create a new File entry for each that is linked to the project
            for file_path in git_history.files {
                let file = NewFile {
                    project_id: project.id,
                    path: file_path,
                }
                .new(self.db)?;

                // 2c. We need to create a FileOwner for each file
                NewFileOwner {
                    sha: sha.clone(),
                    file_id: file.id,
                    owner_id: owner.id,
                    action_date: commit_date,
                }
                .new(self.db)?;
            }
        }
        Ok(())
    }

    fn get_most_recent_commit(&self) -> Option<NaiveDateTime> {
        Commit::fetch_latest_for_project(self.project.id, self.db)
            .map(|c| c.commit_time)
            .ok()
    }
}