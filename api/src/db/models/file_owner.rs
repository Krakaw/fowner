use crate::db::models::extract_all;
use crate::db::Connection;
use crate::errors::FownerError;
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct FileOwner {
    pub file_id: u32,
    pub owner_id: u32,
    pub action_date: NaiveDateTime,
    pub sha: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub struct NewFileOwner {
    pub file_id: u32,
    pub owner_id: u32,
    pub action_date: NaiveDateTime,
    pub sha: String,
}

impl FileOwner {
    pub fn load(
        file_id: u32,
        owner_id: Option<u32>,
        action_date: Option<NaiveDateTime>,
        conn: &Connection,
    ) -> Result<Vec<FileOwner>, FownerError> {
        let mut stmt = conn.prepare("SELECT file_id, owner_id, action_date, sha, created_at, updated_at FROM file_owners WHERE file_id = ?1 AND (?2 IS NULL OR owner_id = ?2) AND (?3 IS NULL or action_date = ?3) ORDER BY action_date DESC")?;
        extract_all!(
            params![file_id, owner_id, action_date.map(|d| d.timestamp())],
            stmt
        )
    }
}

impl<'stmt> From<&Row<'stmt>> for FileOwner {
    fn from(row: &Row) -> Self {
        Self {
            file_id: row.get(0).unwrap(),
            owner_id: row.get(1).unwrap(),
            action_date: NaiveDateTime::from_timestamp(row.get(2).unwrap(), 0),
            sha: row.get(3).unwrap(),
            created_at: NaiveDateTime::from_timestamp(row.get(4).unwrap(), 0),
            updated_at: NaiveDateTime::from_timestamp(row.get(5).unwrap(), 0),
        }
    }
}

impl NewFileOwner {
    pub fn save(&self, conn: &Connection) -> Result<FileOwner, FownerError> {
        let mut stmt = conn.prepare("INSERT INTO file_owners (file_id, owner_id, action_date, sha, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, strftime('%s','now'), strftime('%s','now'))")?;
        let _res = stmt.execute(params![
            self.file_id,
            self.owner_id,
            self.action_date.timestamp(),
            self.sha
        ])?;
        let file_owner = FileOwner::load(
            self.file_id,
            Some(self.owner_id),
            Some(self.action_date),
            conn,
        )?;
        file_owner
            .first()
            .cloned()
            .ok_or_else(|| FownerError::NotFound("File Owner not found".to_string()))
    }
}
