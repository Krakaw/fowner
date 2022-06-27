use crate::db::models::{extract_all, extract_first};
use crate::errors::FownerError;
use crate::{Db, GitRepo};
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};
use serde::{Deserialize, Serialize};
use std::fs;
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

pub struct NewProject {
    pub name: Option<String>,
    pub repo_url: Option<String>,
    pub path: PathBuf,
}
impl NewProject {
    pub fn save(&self, db: &Db) -> Result<Project, FownerError> {
        if let Ok(project) = Project::load_by_path(&self.path, db) {
            return Ok(project);
        }
        let absolute = fs::canonicalize(self.path.clone())?;
        let absolute = absolute.to_string_lossy();
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("INSERT INTO projects (name, repo_url, path, created_at, updated_at) VALUES (?, ?, ?, strftime('%s','now'), strftime('%s','now'))")?;
        let _res = stmt.execute(params![self.name, self.repo_url, absolute])?;
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

    pub fn load(id: u32, db: &Db) -> Result<Self, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, name, repo_url, path, created_at, updated_at FROM projects WHERE id = ?1;",
        )?;

        extract_first!(params![id], stmt)
    }

    /// Loads by an exact path match
    pub fn load_by_path(path: &Path, db: &Db) -> Result<Self, FownerError> {
        let absolute = fs::canonicalize(path)?;
        let absolute = absolute.to_string_lossy();
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, name, repo_url, path, created_at, updated_at FROM projects WHERE LOWER(path) = LOWER(?);")?;
        let rows = stmt.query_row(params![absolute], |r| Ok(Self::from(r)))?;
        Ok(rows)
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

impl From<&GitRepo> for NewProject {
    fn from(repo: &GitRepo) -> Self {
        NewProject {
            name: repo.name.clone(),
            repo_url: repo.url.clone(),
            path: repo.path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::db::models::project::NewProject;
    use crate::test::tests::init;
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
        let (db, tmp_dir) = init();
        let project1 = add_project(&db, &tmp_dir, "Project_1".to_string());
        let project2 = add_project(&db, &tmp_dir, "Project_2".to_string());
        let db_projects = Project::all(&db).unwrap();
        assert_eq!(db_projects.len(), 2);
        assert_eq!(project1, db_projects[0]);
        assert_eq!(project2, db_projects[1]);
    }

    #[test]
    fn load() {
        let (db, tmp_dir) = init();
        let _project1 = add_project(&db, &tmp_dir, "Project_1".to_string());
        let project2 = add_project(&db, &tmp_dir, "Project_2".to_string());
        let db_projects = Project::load(2, &db).unwrap();
        assert_eq!(project2, db_projects);
    }

    #[test]
    fn load_by_path() {
        let (db, tmp_dir) = init();
        let project1 = add_project(&db, &tmp_dir, "Project_1".to_string());
        let _project2 = add_project(&db, &tmp_dir, "Project_2".to_string());
        let db_projects = Project::load_by_path(tmp_dir.join("Project_1").as_path(), &db).unwrap();
        assert_eq!(project1, db_projects);
        // Load non existent
        let not_found_db_projects = Project::load_by_path(tmp_dir.join("Project_x").as_path(), &db);
        assert!(not_found_db_projects.is_err());
    }
}
