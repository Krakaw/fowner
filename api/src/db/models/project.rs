use crate::db::models::feature::Feature;
use crate::db::models::{extract_all, extract_first};
use crate::errors::FownerError;
use crate::git::manager::GitManager;
use crate::{Db, File};
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub id: u32,
    pub name: Option<String>,
    pub repo_url: Option<String>,
    pub path: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NewProject {
    pub name: Option<String>,
    pub repo_url: Option<String>,
    pub path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayProject {
    pub project: Project,
    pub features: Vec<Feature>,
    pub files: Vec<File>,
}

impl NewProject {
    pub fn save(&self, db: &Db) -> Result<Project, FownerError> {
        if let Ok(project) = Project::load_by_path(&self.path, db) {
            return Ok(project);
        }
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("INSERT INTO projects (name, repo_url, path, created_at, updated_at) VALUES (?, ?, ?, strftime('%s','now'), strftime('%s','now'))")?;
        let _res = stmt.execute(params![
            self.name,
            self.repo_url,
            self.path.to_string_lossy()
        ])?;
        let id = conn.last_insert_rowid();
        Project::load(id as u32, db)
    }
}

impl Project {
    pub fn all(db: &Db) -> Result<Vec<Self>, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt =
            conn.prepare("SELECT id, name, repo_url, path, created_at, updated_at FROM projects;")?;
        extract_all!(params![], stmt)
    }

    pub fn get_absolute_dir(&self, storage_path: &Path) -> PathBuf {
        let db_path = PathBuf::from(self.path.clone());
        if db_path.is_absolute() {
            return db_path;
        }
        storage_path.join(db_path)
    }

    pub fn load(id: u32, db: &Db) -> Result<Self, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, repo_url, path, created_at, updated_at FROM projects WHERE id = ?1;",
        )?;

        extract_first!(params![id], stmt)
    }

    /// Loads by an exact path match
    pub fn load_by_path(path: &Path, db: &Db) -> Result<Self, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, name, repo_url, path, created_at, updated_at FROM projects WHERE LOWER(path) LIKE ?1 LIMIT 1;")?;
        let path = format!("%{}", path.to_string_lossy());
        let rows = stmt.query_row(params![path], |r| Ok(Self::from(r)))?;
        Ok(rows)
    }

    pub fn for_display(&self, db: &Db) -> Result<DisplayProject, FownerError> {
        let features = Feature::load_by_project(self.id, db)?;
        let files = File::all(self.id, db)?;
        Ok(DisplayProject {
            project: self.clone(),
            features,
            files,
        })
    }
}

impl<'stmt> From<&Row<'stmt>> for Project {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            name: row.get(1).unwrap(),
            repo_url: row.get(2).unwrap(),
            path: row.get(3).unwrap(),
            created_at: NaiveDateTime::from_timestamp(row.get(4).unwrap(), 0),
            updated_at: NaiveDateTime::from_timestamp(row.get(5).unwrap(), 0),
        }
    }
}

impl From<&GitManager> for NewProject {
    fn from(repo: &GitManager) -> Self {
        NewProject {
            name: None,
            repo_url: repo.url.clone(),
            path: repo.path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::models::project::NewProject;
    use crate::test::tests::TestHandler;
    use crate::{Db, Project};
    use std::path::Path;

    fn add_project(db: &Db, tmp_dir: &Path, name: String) -> Project {
        let buf = tmp_dir.join(&name);
        if !buf.exists() {
            std::fs::create_dir(buf.clone()).unwrap();
        }

        NewProject {
            name: Some(name),
            repo_url: None,
            path: buf,
        }
        .save(db)
        .unwrap()
    }

    #[test]
    fn all() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let tmp_dir = &handler.tmp_dir;
        let project1 = add_project(db, tmp_dir, "Project_1".to_string());
        let project2 = add_project(db, tmp_dir, "Project_2".to_string());
        let db_projects = Project::all(db).unwrap();
        assert_eq!(db_projects.len(), 2);
        assert_eq!(project1, db_projects[0]);
        assert_eq!(project2, db_projects[1]);
    }

    #[test]
    fn load() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let tmp_dir = &handler.tmp_dir;
        let _project1 = add_project(db, tmp_dir, "Project_1".to_string());
        let project2 = add_project(db, tmp_dir, "Project_2".to_string());
        let db_projects = Project::load(2, db).unwrap();
        assert_eq!(project2, db_projects);
    }

    #[test]
    fn load_by_path() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let tmp_dir = &handler.tmp_dir;
        let project1 = add_project(db, tmp_dir, "Project_1".to_string());
        let _project2 = add_project(db, tmp_dir, "Project_2".to_string());
        let db_projects = Project::load_by_path(tmp_dir.join("Project_1").as_path(), db).unwrap();
        assert_eq!(project1, db_projects);
        // Load non existent
        let not_found_db_projects = Project::load_by_path(tmp_dir.join("Project_x").as_path(), db);
        assert!(not_found_db_projects.is_err());
    }
}
