use crate::Db;
use anyhow::Result;
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

pub struct NewCommit {}

impl Commit {
    pub fn fetch_latest(db: &Db) -> Result<Self> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("SELECT * FROM commits ORDER BY commit_time DESC LIMIT 1;")?;
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
