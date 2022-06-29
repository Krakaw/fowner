use crate::db::models::commit::{Commit, NewCommit};
use crate::db::models::feature::NewFeature;
use crate::db::models::file::NewFile;
use crate::db::models::file_commit::FileCommit;
use crate::db::models::file_feature::NewFileFeature;
use crate::db::models::file_owner::NewFileOwner;
use crate::db::models::owner::NewOwner;
use crate::db::models::project::{NewProject, Project};
use crate::errors::FownerError;
use crate::{Db, GitRepo};
use chrono::NaiveDateTime;
use log::debug;

pub struct Processor<'a> {
    pub db: &'a Db,
    pub repo: GitRepo,
    pub project: Project,
}

impl<'a> Processor<'a> {
    pub fn new(repo: GitRepo, db: &'a Db) -> Result<Self, FownerError> {
        let project = NewProject::from(&repo).save(db)?;
        Ok(Processor { db, repo, project })
    }

    pub fn fetch_commits_and_update_db(&mut self) -> Result<usize, FownerError> {
        let number_of_commits = self.fetch_history_and_store_data()?;
        Ok(number_of_commits)
    }

    pub fn fetch_history_and_store_data(&mut self) -> Result<usize, FownerError> {
        let latest_commit = self.get_most_recent_commit();
        let history = self.repo.parse(latest_commit)?;
        let project = self.project.clone();
        let project_id = project.id;
        let number_of_commits = history.len();
        debug!("{} new commits to process", number_of_commits);
        for git_history in history {
            // For each GitHistory
            // 1. We need to create an Owner from the handle
            let owner = NewOwner {
                handle: git_history.handle,
                name: None,
                primary_owner_id: None,
            }
            .save(self.db)?;
            // 2. We need to create a Commit for the hash
            let commit_date = NaiveDateTime::from_timestamp(git_history.timestamp as i64, 0);
            let sha = git_history.hash.clone();
            let commit = NewCommit {
                project_id,
                sha: sha.clone(),
                description: git_history.summary.clone(),
                commit_time: commit_date,
            }
            .save(self.db)?;
            // 3. Create the features
            let mut features = vec![];
            for feature in git_history.features {
                features.push(
                    NewFeature {
                        project_id,
                        name: feature,
                        description: None,
                    }
                    .save(self.db)?,
                );
            }
            // 4a. We need to extract all of the files and create a new File entry for each that is linked to the project
            for file_path in git_history.files {
                let file = NewFile {
                    project_id: project.id,
                    path: file_path,
                }
                .save(self.db)?;

                // 4b. We need to create a FileOwner for each file
                NewFileOwner {
                    sha: sha.clone(),
                    file_id: file.id,
                    owner_id: owner.id,
                    action_date: commit_date,
                }
                .save(self.db)?;

                // 4c. We create a FileCommit link for every file
                FileCommit {
                    file_id: file.id,
                    commit_id: commit.id,
                }
                .save(self.db)?;

                // 4d. Attach the features to the files
                for feature in &features {
                    NewFileFeature {
                        file_id: file.id,
                        feature_id: feature.id,
                    }
                    .save(self.db)?;
                }
            }
        }
        Ok(number_of_commits)
    }

    fn get_most_recent_commit(&self) -> Option<NaiveDateTime> {
        Commit::fetch_latest_for_project(self.project.id, self.db)
            .map(|c| c.commit_time)
            .ok()
    }
}

#[cfg(test)]
mod tests {}
