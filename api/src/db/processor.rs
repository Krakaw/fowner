use chrono::NaiveDateTime;
use log::{debug, error, trace};

use crate::db::models::commit::{Commit, NewCommit};
use crate::db::models::feature::NewFeature;
use crate::db::models::file::NewFile;
use crate::db::models::file_commit::FileCommit;
use crate::db::models::file_owner::NewFileOwner;
use crate::db::models::owner::NewOwner;
use crate::db::models::project::{NewProject, Project};
use crate::db::Connection;
use crate::errors::FownerError;
use crate::git::github::Github;
use crate::git::manager::GitManager;

pub struct Processor<'a> {
    pub conn: &'a Connection<'a>,
    pub git_manager: GitManager,
    pub project: Project,
}

impl<'a> Processor<'a> {
    pub fn new(git_manager: GitManager, conn: &'a Connection) -> Result<Self, FownerError> {
        let project = NewProject::from(&git_manager).save_or_load(conn)?;
        Ok(Processor {
            conn,
            git_manager,
            project,
        })
    }

    pub async fn fetch_commits_and_update_db(
        &self,
        stop_at_sha: Option<String>,
        skip_github_labels: bool,
    ) -> Result<(usize, usize), FownerError> {
        let number_of_commits = self
            .fetch_history_and_store_data(stop_at_sha, skip_github_labels)
            .await?;
        Ok(number_of_commits)
    }

    pub async fn fetch_history_and_store_data(
        &self,
        stop_at_sha: Option<String>,
        skip_github_labels: bool,
    ) -> Result<(usize, usize), FownerError> {
        let latest_commit = self.get_most_recent_commit();
        let history = self.git_manager.parse_history(latest_commit)?;
        let project = self.project.clone();
        let project_id = project.id;
        let number_of_commits = history.len();
        let github = Github::try_from(&project).ok();
        let stop_at_sha = stop_at_sha.unwrap_or_default();
        debug!("{} new commits to process", number_of_commits);

        let mut commit_counter = 1;
        for git_history in history {
            // For each GitHistory
            // 1. We need to create an Owner from the handle

            let owner = NewOwner {
                handle: git_history.handle,
                name: None,
                primary_owner_id: None,
            }
            .save_or_load(self.conn)?;

            // 2. We need to create a Commit for the hash
            let commit_date = NaiveDateTime::from_timestamp(git_history.timestamp as i64, 0);
            let sha = git_history.sha.clone();
            if sha == stop_at_sha {
                break;
            }
            let new_commit = NewCommit {
                owner_id: owner.id,
                project_id,
                sha: sha.clone(),
                parent_sha: git_history.parent_sha.clone(),
                description: git_history.summary.clone(),
                commit_time: commit_date,
            };
            trace!("new_commit = {:?}", new_commit);
            let commit = new_commit.save(self.conn)?;
            // 3. Create the features
            let mut features = vec![];
            let mut source_feature_names = vec![];
            // Use the github tags if they're available as the primary features
            if !skip_github_labels {
                if let Some(github) = &github {
                    debug!("Fetching github labels");
                    match github.fetch_labels_for_commit(sha.as_str()).await {
                        Ok(mut labels) => {
                            debug!("Found labels {:?}", labels);
                            source_feature_names.append(&mut labels);
                        }
                        Err(e) => {
                            error!("{:?}", e);
                        }
                    }
                }
            }
            // If there were not github tags, or no repo to pull from then use the git source commit messages
            if source_feature_names.is_empty() && !project.github_labels_only {
                source_feature_names.append(&mut git_history.features.clone());
            }
            for feature in source_feature_names {
                features.push(
                    NewFeature {
                        project_id,
                        name: feature,
                        description: None,
                    }
                    .save(self.conn)?,
                );
            }
            // 4a. We need to extract all of the files and create a new File entry for each that is linked to the project
            for file_path in git_history.files {
                let file = NewFile {
                    project_id: project.id,
                    path: file_path,
                    no_features: false,
                }
                .save(self.conn)?;

                // 4b. We need to create a FileOwner for each file
                NewFileOwner {
                    sha: sha.clone(),
                    file_id: file.id,
                    owner_id: owner.id,
                    action_date: commit_date,
                }
                .save(self.conn)?;

                // 4c. We create a FileCommit link for every file
                FileCommit {
                    file_id: file.id,
                    commit_id: commit.id,
                }
                .save(self.conn)?;

                // 4d. Attach the features to the files
                for feature in &features {
                    let _f = file.add_feature(feature.id, self.conn);
                }
            }
            debug!(
                "Processed {} of {} - {}",
                commit_counter,
                number_of_commits,
                sha.chars().into_iter().take(7).collect::<String>()
            );
            commit_counter += 1;
        }
        debug!("Finished processing project id: {}", project_id);
        Ok((commit_counter - 1, number_of_commits))
    }

    fn get_most_recent_commit(&self) -> Option<NaiveDateTime> {
        Commit::fetch_latest_for_project(self.project.id, self.conn)
            .map(|c| c.commit_time)
            .ok()
    }
}

#[cfg(test)]
mod tests {}
