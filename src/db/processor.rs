use crate::db::models::commit::Commit;
use crate::db::models::file::{File, NewFile};
use crate::db::models::owner::NewOwner;
use crate::db::models::project::{NewProject, Project};
use crate::{Db, GitRepo};
use anyhow::{anyhow, Result};

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
        let history = self.repo.parse(latest_commit as usize)?;
        for row in history.clone() {
            // For each GitHistory
            // 1. We need to create an Owner from the handle
            let owner = NewOwner {
                handle: row.handle,
                name: None,
            }
            .new(self.db)?;

            // 2a. We need to extract all of the files and create a new File entry for each that is linked to the project
            for file_path in row.files {
                let file = NewFile {
                    project_id: self.project.unwrap().id,
                    path: file_path,
                }
                .new(self.db)?;
                // 2b. We need to create a Commit for the hash for each file entry
            }

            // 2c. We need to create a FileOwner for each file

            // Create the owner
        }
        Ok(())
    }

    fn get_most_recent_commit(&self) -> i64 {
        Commit::fetch_latest(self.db)
            .map(|c| c.commit_time.timestamp())
            .unwrap_or_else(|_| 0_i64)
    }
}
