use crate::db::models::project::NewProject;
use crate::{Connection, FownerError, Project};
use std::env::temp_dir;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub struct ProjectBuilder {
    pub name: Option<String>,
    pub repo_url: Option<String>,
    pub path: PathBuf,
    pub github_api_token: Option<String>,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl Default for ProjectBuilder {
    fn default() -> Self {
        let path = temp_dir();
        Self {
            name: None,
            repo_url: None,
            github_api_token: None,
            path,
            __non_exhaustive: (),
        }
    }
}

#[allow(dead_code)]
impl ProjectBuilder {
    pub fn with_path(path: &Path) -> ProjectBuilder {
        Self {
            path: path.to_path_buf(),
            ..ProjectBuilder::default()
        }
    }

    pub fn build(self, conn: &Connection) -> Result<Project, FownerError> {
        NewProject {
            name: self.name,
            repo_url: self.repo_url,
            path: self.path,
            github_api_token: self.github_api_token,
        }
        .save(conn)
    }
}
