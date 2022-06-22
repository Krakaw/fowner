use crate::{Db, GitRepo};
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
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
    pub fn save(&self, db: &Db) -> Result<Project> {
        if let Ok(project) = Project::load_by_path(&self.path, db) {
            return Ok(project);
        }
        let absolute = fs::canonicalize(self.path.clone())?;
        let absolute = absolute.to_string_lossy();
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("INSERT INTO projects (name, repo_url, path, created_at, updated_at) VALUES (?, ?, ?, strftime('%s','now'), strftime('%s','now'))")?;
        let _res = stmt.execute(params![self.name, self.repo_url, absolute,])?;
        Project::load_by_path(&self.path, db)
    }
}

impl Project {
    pub fn load_by_path(path: &Path, db: &Db) -> Result<Self> {
        let absolute = fs::canonicalize(path)?;
        let absolute = absolute.to_string_lossy();
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, name, repo_url, path, created_at, updated_at FROM projects WHERE LOWER(path) = LOWER(?);")?;
        let mut rows = stmt.query(params![absolute])?;
        if let Some(row) = rows.next()? {
            Ok(Project::from(row))
        } else {
            Err(anyhow!("Project not found"))
        }
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
