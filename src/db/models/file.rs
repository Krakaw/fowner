use crate::Db;
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};

pub struct File {
    pub id: u32,
    pub project_id: u32,
    pub path: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl File {
    pub fn load_by_path(project_id: u32, path: String, db: &Db) -> Result<File> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, project_id, path, created_at, updated_at FROM files WHERE LOWER(path) = LOWER(?);")?;
        let mut rows = stmt.query(params![project_id, path])?;
        if let Some(row) = rows.next()? {
            Ok(File::from(row))
        } else {
            Err(anyhow!("File not found"))
        }
    }
}
pub struct NewFile {
    pub project_id: u32,
    pub path: String,
}

impl NewFile {
    pub fn new(&self, db: &Db) -> Result<File> {
        if let Ok(file) = File::load_by_path(self.project_id, self.path.clone(), db) {
            return Ok(file);
        };
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("INSERT INTO files (project_id, path, created_at, updated_at) VALUES (?, ?, strftime('%s','now'), strftime('%s','now'))")?;
        let _res = stmt.execute(params![self.project_id.clone(), self.path.clone()])?;
        File::load_by_path(self.project_id, self.path.clone(), db)
    }
}

impl<'stmt> From<&Row<'stmt>> for File {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            project_id: row.get(1).unwrap(),
            path: row.get(2).unwrap(),
            created_at: NaiveDateTime::from_timestamp(row.get(3).unwrap(), 0),
            updated_at: NaiveDateTime::from_timestamp(row.get(4).unwrap(), 0),
        }
    }
}
