use crate::db::models::commit::NewCommit;
use crate::db::models::feature::NewFeature;
use crate::db::models::file::NewFile;
use crate::db::models::file_commit::FileCommit;
use crate::db::models::file_feature::NewFileFeature;
use crate::db::models::project::NewProject;
use crate::{Connection, File, FownerError, Project};
use chrono::{NaiveDateTime, Utc};
use std::env::temp_dir;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct FileBuilder {
    pub project_id: u32,
    pub path: String,
    pub features: Vec<String>,
    pub commits: Vec<String>,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl Default for FileBuilder {
    fn default() -> Self {
        let path = temp_dir();
        Self {
            project_id: 0,
            path: path.to_string_lossy().to_string(),
            features: vec![],
            commits: vec![],
            __non_exhaustive: (),
        }
    }
}

#[allow(dead_code)]
impl FileBuilder {
    pub fn with_features(self, features: Vec<String>) -> Self {
        Self { features, ..self }
    }

    pub fn with_commits(self, commits: Vec<String>) -> Self {
        Self { commits, ..self }
    }

    pub fn build(self, conn: &Connection) -> Result<File, FownerError> {
        let file = NewFile {
            project_id: self.project_id,
            path: self.path,
        }
        .save(conn)
        .unwrap();

        for feature_name in self.features {
            let feature = NewFeature {
                project_id: self.project_id,
                name: feature_name,
                description: None,
            }
            .save(&conn)
            .unwrap();
            NewFileFeature {
                file_id: file.id,
                feature_id: feature.id,
            }
            .save(&conn)
            .unwrap();
        }

        let mut last_sha = None;
        for commit_sha in self.commits {
            let commit = NewCommit {
                project_id: self.project_id,
                sha: commit_sha.clone(),
                parent_sha: last_sha.clone(),
                description: commit_sha.clone(),
                commit_time: Utc::now().naive_utc(),
            }
            .save(&conn)
            .unwrap();
            last_sha = Some(vec![commit_sha]);
            FileCommit {
                file_id: file.id,
                commit_id: commit.id,
            }
            .save(&conn)
            .unwrap();
        }
        Ok(file)
    }
}
