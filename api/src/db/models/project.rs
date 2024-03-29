use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

use crate::db::models::feature::Feature;
use crate::db::models::{extract_all, extract_first};
use crate::db::Connection;
use crate::errors::FownerError;
use crate::git::manager::GitManager;
use crate::File;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub id: u32,
    pub name: Option<String>,
    pub repo_url: Option<String>,
    pub github_api_token: Option<String>,
    pub github_labels_only: bool,
    pub path: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NewProject {
    pub name: Option<String>,
    pub repo_url: Option<String>,
    pub github_api_token: Option<String>,
    pub github_labels_only: bool,
    pub path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayProject {
    pub project: Project,
    pub features: Vec<Feature>,
    pub files: Vec<File>,
}

impl NewProject {
    pub fn save_or_load(&self, conn: &Connection) -> Result<Project, FownerError> {
        if let Ok(project) = Project::load_by_path(&self.path, conn) {
            return Ok(project);
        }
        self.save(conn)
    }
    pub fn save(&self, conn: &Connection) -> Result<Project, FownerError> {
        let mut stmt = conn.prepare(
            r#"
        INSERT INTO projects (name, repo_url, github_api_token, github_labels_only, path, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, strftime('%s', 'now'), strftime('%s', 'now'))
        "#,
        )?;
        let _res = stmt.execute(params![
            self.name,
            self.repo_url,
            self.github_api_token,
            self.github_labels_only,
            self.path.to_string_lossy()
        ])?;
        let id = conn.last_insert_rowid();
        Project::load(id as u32, conn)
    }
}

impl Project {
    pub fn sql(where_clause: Option<String>, limit_clause: Option<String>) -> String {
        format!(
            r#"
            SELECT
                id, name, repo_url, github_api_token, github_labels_only, path, created_at, updated_at
                FROM projects
                {}
                {}
        "#,
            where_clause.unwrap_or_default(),
            limit_clause.unwrap_or_default()
        )
    }
    pub fn all(conn: &Connection) -> Result<Vec<Self>, FownerError> {
        let mut stmt = conn.prepare(&Project::sql(None, None))?;
        extract_all!(params![], stmt)
    }

    pub fn get_absolute_dir(
        &self,
        storage_path: &Path,
        create_missing: bool,
    ) -> Result<PathBuf, FownerError> {
        let db_path = PathBuf::from(self.path.clone());
        let result_path = if db_path.is_absolute() {
            db_path
        } else {
            storage_path.join(db_path)
        };
        if create_missing && !result_path.exists() {
            std::fs::create_dir_all(&result_path)?;
        }
        Ok(result_path.canonicalize()?)
    }

    pub fn get_github_api_url(&self) -> Result<String, FownerError> {
        if let Some(repo_url) = &self.repo_url {
            if !repo_url.contains("github.com") {
                return Err(FownerError::GitError(format!(
                    "Github API url cannot be generated from {}",
                    repo_url
                )));
            }
            let repo_owner = if repo_url.to_lowercase().starts_with("https://") {
                let mut parts: Vec<&str> = repo_url.rsplit('/').collect();
                let repo_owner: Vec<&str> = parts.drain(..2).collect();

                repo_owner
            } else {
                let path: Vec<&str> = repo_url.rsplit(':').collect();
                let repo_owner: Vec<&str> = path
                    .first()
                    .ok_or_else(|| {
                        FownerError::GitError("Invalid SSH Github url in repo_url".to_string())
                    })?
                    .rsplit('/')
                    .collect();
                repo_owner
            };
            let repo = repo_owner
                .first()
                .map(|r| r.replace(".git", ""))
                .ok_or_else(|| {
                    FownerError::GitError("Missing repository in repo_url".to_string())
                })?;
            let owner = repo_owner
                .get(1)
                .ok_or_else(|| FownerError::GitError("Missing owner in repo_url".to_string()))?;

            let github_api_url = format!("https://api.github.com/repos/{}/{}", owner, repo);
            Ok(github_api_url)
        } else {
            Err(FownerError::NotFound(
                "repo_url is missing for this project".to_string(),
            ))
        }
    }

    pub fn load(id: u32, conn: &Connection) -> Result<Self, FownerError> {
        let mut stmt = conn.prepare(&Project::sql(Some("WHERE id = ?1".to_string()), None))?;
        extract_first!(params![id], stmt)
    }

    /// Loads by an exact path match
    pub fn load_by_path(path: &Path, conn: &Connection) -> Result<Self, FownerError> {
        let mut stmt = conn.prepare(&Project::sql(
            Some("WHERE LOWER(path) LIKE ?1".to_string()),
            Some("LIMIT 1".to_string()),
        ))?;
        let path = format!("%{}", path.to_string_lossy());
        let rows = stmt.query_row(params![path], |r| Ok(Self::from(r)))?;
        Ok(rows)
    }

    pub fn for_display(&self, conn: &Connection) -> Result<DisplayProject, FownerError> {
        let features = Feature::load_by_project(self.id, conn)?;
        let files = File::all(self.id, conn)?;
        Ok(DisplayProject {
            project: self.clone(),
            features,
            files,
        })
    }

    pub fn destroy(self, conn: &Connection) -> Result<usize, FownerError> {
        let mut stmt = conn.prepare("DELETE FROM projects WHERE id = ?")?;
        let result = stmt.execute(params![self.id])?;
        Ok(result)
    }
}

impl<'stmt> From<&Row<'stmt>> for Project {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            name: row.get(1).unwrap(),
            repo_url: row.get(2).unwrap(),
            github_api_token: row.get(3).unwrap(),
            github_labels_only: row.get(4).unwrap(),
            path: row.get(5).unwrap(),
            created_at: NaiveDateTime::from_timestamp(row.get(6).unwrap(), 0),
            updated_at: NaiveDateTime::from_timestamp(row.get(7).unwrap(), 0),
        }
    }
}

impl From<&GitManager> for NewProject {
    fn from(repo: &GitManager) -> Self {
        NewProject {
            name: None,
            repo_url: repo.url.clone(),
            path: repo.path.clone(),
            github_api_token: None,
            github_labels_only: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::{Path, PathBuf};

    use crate::db::models::project::NewProject;
    use crate::test::tests::TestHandler;
    use crate::{Connection, Project};

    fn add_project(conn: &Connection, tmp_dir: &Path, name: String) -> Project {
        let path = tmp_dir.join(&name);
        NewProject {
            name: Some(name),
            repo_url: None,
            path,
            github_api_token: None,
            github_labels_only: false,
        }
        .save_or_load(conn)
        .unwrap()
    }

    #[test]
    fn create_by_path_is_unique() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = &Connection::try_from(db).unwrap();

        let tmp_dir = &handler.tmp_dir;
        let project1 = add_project(conn, tmp_dir, "Project_1".to_string());
        let project2 = add_project(conn, tmp_dir, "Project_1".to_string());
        assert_eq!(project1, project2);
        let db_projects = Project::load_by_path(tmp_dir.join("Project_1").as_path(), conn).unwrap();
        assert_eq!(project1, db_projects);
    }

    #[test]
    fn cannot_save_non_unique_projects() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = &Connection::try_from(db).unwrap();

        let tmp_dir = &handler.tmp_dir;
        let project1 = add_project(conn, tmp_dir, "Project_1".to_string());
        let path = tmp_dir.join("Project_1");
        let err_result = NewProject {
            name: Some("Project_1".to_string()),
            repo_url: None,
            path,
            github_api_token: None,
            github_labels_only: false,
        }
        .save(conn);
        eprintln!("err_result = {:?}", err_result);
        assert!(err_result.is_err());
        let db_projects = Project::load_by_path(tmp_dir.join("Project_1").as_path(), conn).unwrap();
        assert_eq!(project1, db_projects);
    }

    #[test]
    fn all() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = &Connection::try_from(db).unwrap();

        let tmp_dir = &handler.tmp_dir;
        let project1 = add_project(conn, tmp_dir, "Project_1".to_string());
        let project2 = add_project(conn, tmp_dir, "Project_2".to_string());
        let db_projects = Project::all(conn).unwrap();
        assert_eq!(db_projects.len(), 2);
        assert_eq!(project1, db_projects[0]);
        assert_eq!(project2, db_projects[1]);
    }

    #[test]
    fn load() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = &Connection::try_from(db).unwrap();
        let tmp_dir = &handler.tmp_dir;
        let _project1 = add_project(conn, tmp_dir, "Project_1".to_string());
        let project2 = add_project(conn, tmp_dir, "Project_2".to_string());
        let db_projects = Project::load(2, conn).unwrap();
        assert_eq!(project2, db_projects);
    }

    #[test]
    fn load_by_path() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = &Connection::try_from(db).unwrap();

        let tmp_dir = &handler.tmp_dir;
        let project1 = add_project(conn, tmp_dir, "Project_1".to_string());
        let _project2 = add_project(conn, tmp_dir, "Project_2".to_string());
        let db_projects = Project::load_by_path(tmp_dir.join("Project_1").as_path(), conn).unwrap();
        assert_eq!(project1, db_projects);
        // Load non existent
        let not_found_db_projects =
            Project::load_by_path(tmp_dir.join("Project_x").as_path(), conn);
        assert!(not_found_db_projects.is_err());
    }

    #[test]
    fn github_token() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = &Connection::try_from(db).unwrap();
        let tmp_dir = &handler.tmp_dir;
        NewProject {
            name: Some("Project 1".to_string()),
            repo_url: None,
            path: tmp_dir.to_path_buf(),
            github_api_token: Some("abc".to_string()),
            github_labels_only: false,
        }
        .save(conn)
        .unwrap();
        let db_project = Project::load(1, conn).unwrap();
        assert_eq!(db_project.github_api_token, Some("abc".to_string()));
    }

    #[test]
    fn get_github_api_url() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = &Connection::try_from(db).unwrap();

        let tmp_dir = &handler.tmp_dir;
        let project = NewProject {
            name: Some("Project 1".to_string()),
            repo_url: Some("https://github.com/Krakaw/fowner.git".to_string()),
            path: tmp_dir.join("p1"),
            github_api_token: Some("abc".to_string()),
            github_labels_only: false,
        }
        .save(conn)
        .unwrap();
        let gh_api_url = project.get_github_api_url().unwrap();
        assert_eq!(gh_api_url, "https://api.github.com/repos/Krakaw/fowner");

        let project2 = NewProject {
            name: Some("Project 2".to_string()),
            repo_url: Some("git@github.com:Krakaw/fowner.git".to_string()),
            path: tmp_dir.join("p2"),
            github_api_token: Some("abc".to_string()),
            github_labels_only: false,
        }
        .save(conn)
        .unwrap();
        let gh_api_url = project2.get_github_api_url().unwrap();
        assert_eq!(gh_api_url, "https://api.github.com/repos/Krakaw/fowner");

        let project3 = NewProject {
            name: Some("Project 3".to_string()),
            repo_url: Some("git@github.com:tari-labs/emoji.id-frontend.git".to_string()),
            path: tmp_dir.join("p3"),
            github_api_token: Some("abc".to_string()),
            github_labels_only: false,
        }
        .save(conn)
        .unwrap();
        eprintln!("project3 = {:?}", project3);
        let gh_api_url = project3.get_github_api_url().unwrap();
        assert_eq!(
            gh_api_url,
            "https://api.github.com/repos/tari-labs/emoji.id-frontend"
        );
    }

    #[test]
    fn get_absolute_dir() {
        let handler = TestHandler::init();
        let db = &handler.db;

        let conn = Connection::try_from(db).unwrap();

        let project = NewProject {
            name: Some("Project 1".to_string()),
            repo_url: Some("https://github.com/Krakaw/fowner.git".to_string()),
            path: PathBuf::from("data/fowner".to_string()),
            github_api_token: Some("abc".to_string()),
            github_labels_only: false,
        }
        .save(&conn)
        .unwrap();

        let current_dir = env::current_dir().unwrap();
        let current_dir = current_dir.to_str().unwrap();
        let absolute = project
            .get_absolute_dir(PathBuf::from("./sources").as_path(), true)
            .unwrap();

        let current_absolute = format!("{}/sources/data/fowner", current_dir);
        assert_eq!(current_absolute, absolute.to_str().unwrap());

        let absolute = project
            .get_absolute_dir(
                PathBuf::from(format!("{}/sources", current_dir)).as_path(),
                false,
            )
            .unwrap();

        assert_eq!(current_absolute, absolute.to_str().unwrap());
        let absolute_project = NewProject {
            name: Some("Project 2".to_string()),
            repo_url: Some("https://github.com/Krakaw/fowner.git".to_string()),
            path: handler.tmp_dir.clone(),
            github_api_token: Some("abc".to_string()),
            github_labels_only: false,
        }
        .save(&conn)
        .unwrap();
        assert!(absolute_project
            .get_absolute_dir(PathBuf::from("./sources").as_path(), false)
            .unwrap()
            .to_str()
            .unwrap()
            .contains(handler.tmp_dir.to_str().unwrap()));
    }
}
