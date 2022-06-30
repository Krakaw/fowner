use crate::db::models::project::NewProject;
use crate::{Db, FownerError, Project};
use std::env::temp_dir;
use std::path::PathBuf;

#[allow(dead_code)]
pub struct ProjectBuilder {
    pub name: Option<String>,
    pub repo_url: Option<String>,
    pub path: PathBuf,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl Default for ProjectBuilder {
    fn default() -> Self {
        let path = temp_dir();
        Self {
            name: None,
            repo_url: None,
            path,
            __non_exhaustive: (),
        }
    }
}

#[allow(dead_code)]
impl ProjectBuilder {
    pub fn with_path(path: PathBuf) -> ProjectBuilder {
        Self {
            path,
            ..ProjectBuilder::default()
        }
    }

    pub fn build(self, db: &Db) -> Result<Project, FownerError> {
        NewProject {
            name: self.name,
            repo_url: self.repo_url,
            path: self.path,
        }
        .save(db)
    }
}
