use crate::db::models::commit::{Commit, NewCommit};
use crate::db::models::feature::NewFeature;
use crate::db::models::file::NewFile;
use crate::db::models::file_commit::FileCommit;
use crate::db::models::file_feature::NewFileFeature;
use crate::db::models::file_owner::NewFileOwner;
use crate::db::models::owner::NewOwner;
use crate::db::models::project::{NewProject, Project};
use crate::errors::FownerError;
use crate::git::github::Github;
use crate::git::manager::GitManager;
use crate::Db;
use chrono::NaiveDateTime;
use log::debug;

pub struct Processor<'a> {
    pub db: &'a Db,
    pub git_manager: GitManager,
    pub project: Project,
}

impl<'a> Processor<'a> {
    pub fn new(git_manager: GitManager, db: &'a Db) -> Result<Self, FownerError> {
        let project = NewProject::from(&git_manager).save(db)?;
        Ok(Processor {
            db,
            git_manager,
            project,
        })
    }

    pub async fn fetch_commits_and_update_db(
        &self,
        stop_at_sha: Option<String>,
    ) -> Result<usize, FownerError> {
        let number_of_commits = self.fetch_history_and_store_data(stop_at_sha).await?;
        Ok(number_of_commits)
    }

    pub async fn fetch_history_and_store_data(
        &self,
        stop_at_sha: Option<String>,
    ) -> Result<usize, FownerError> {
        let latest_commit = self.get_most_recent_commit();
        let history = self.git_manager.parse_history(latest_commit)?;
        let project = self.project.clone();
        let project_id = project.id;
        let number_of_commits = history.len();
        let github = Github::try_from(&project).ok();
        let stop_at_sha = stop_at_sha.unwrap_or_default();
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
            let sha = git_history.sha.clone();
            if sha == stop_at_sha {
                break;
            }
            let commit = NewCommit {
                project_id,
                sha: sha.clone(),
                parent_sha: git_history.parent_sha.clone(),
                description: git_history.summary.clone(),
                commit_time: commit_date,
            }
            .save(self.db)?;
            // 3. Create the features
            let mut features = vec![];
            let mut source_feature_names = vec![];
            // Use the github tags if they're available as the primary features
            if let Some(github) = &github {
                if let Ok(mut labels) = github.fetch_labels_for_commit(sha.as_str()).await {
                    source_feature_names.append(&mut labels);
                }
            }
            // If there were not github tags, or no repo to pull from then use the git source commit messages
            if source_feature_names.is_empty() {
                source_feature_names.append(&mut git_history.features.clone());
            }
            for feature in source_feature_names {
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
