use crate::Db;
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};
pub struct Commit {
    pub id: u32,
    pub file_id: u32,
    pub sha: String,
    pub description: String,
    pub commit_time: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub struct NewCommit {
    pub file_id: u32,
    pub sha: String,
    pub description: String,
    pub commit_time: NaiveDateTime,
}

impl NewCommit {
    pub fn new(&self, db: &Db) -> Result<Commit> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("INSERT INTO commits (file_id, sha, description, commit_time, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, strftime('%s','now'), strftime('%s','now'))")?;
        let _res = stmt.execute(params![
            self.file_id,
            self.sha,
            self.description,
            self.commit_time.timestamp()
        ])?;
        let id = conn.last_insert_rowid();
        Commit::load(id, db)
    }
}
impl Commit {
    pub fn load(id: i64, db: &Db) -> Result<Self> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, file_id, sha, description, commit_time, created_at, updated_at FROM commits WHERE id = ?1;")?;
        let mut results = stmt.query(params![id])?;
        let result = results
            .next()?
            .map(Commit::from)
            .ok_or(anyhow!("Failed to fetch commit"))?;
        Ok(result)
    }
    pub fn fetch_latest(db: &Db) -> Result<Self> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, file_id, sha, description, commit_time, created_at, updated_at FROM commits ORDER BY commit_time DESC LIMIT 1;")?;
        let mut results = stmt.query(params![])?;
        while let Some(row) = results.next()? {
            return Ok(Commit::from(row));
        }
        Err(anyhow::anyhow!("No commits found"))
    }
}

impl<'stmt> From<&Row<'stmt>> for Commit {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            file_id: row.get(1).unwrap(),
            sha: row.get(2).unwrap(),
            description: row.get(3).unwrap(),
            commit_time: NaiveDateTime::from_timestamp(row.get(4).unwrap(), 0),
            created_at: NaiveDateTime::from_timestamp(row.get(5).unwrap(), 0),
            updated_at: NaiveDateTime::from_timestamp(row.get(6).unwrap(), 0),
        }
    }
}
