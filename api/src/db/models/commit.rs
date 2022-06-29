use crate::db::models::extract_first;
use crate::errors::FownerError;
use crate::Db;
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};

pub struct Commit {
    pub id: u32,
    pub project_id: u32,
    pub sha: String,
    pub description: String,
    pub commit_time: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct NewCommit {
    pub project_id: u32,
    pub sha: String,
    pub description: String,
    pub commit_time: NaiveDateTime,
}

impl NewCommit {
    pub fn save(&self, db: &Db) -> Result<Commit, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("INSERT INTO commits (project_id, sha, description, commit_time, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, strftime('%s','now'), strftime('%s','now'))")?;
        let _res = stmt.execute(params![
            self.project_id,
            self.sha,
            self.description,
            self.commit_time.timestamp()
        ])?;
        let id = conn.last_insert_rowid();
        Commit::load(id, db)
    }
}
impl Commit {
    fn sql(where_clause: String) -> String {
        format!("SELECT id, project_id, sha, description, commit_time, created_at, updated_at FROM commits {}", where_clause)
    }
    pub fn load_by_sha(sha: String, db: &Db) -> Result<Self, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare(&Commit::sql("WHERE sha LIKE ?1;".to_string()))?;
        extract_first!(params![&format!("{}%", sha)], stmt)
    }

    pub fn load(id: i64, db: &Db) -> Result<Self, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare(&Commit::sql("WHERE id = ?1;".to_string()))?;
        extract_first!(params![id], stmt)
    }
    pub fn fetch_latest_for_project(project_id: u32, db: &Db) -> Result<Self, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, project_id, sha, description, commit_time, created_at, updated_at FROM commits WHERE project_id = ?1 ORDER BY commit_time DESC LIMIT 1;")?;
        extract_first!(params![project_id], stmt)
    }
}

impl<'stmt> From<&Row<'stmt>> for Commit {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            project_id: row.get(1).unwrap(),
            sha: row.get(2).unwrap(),
            description: row.get(3).unwrap(),
            commit_time: NaiveDateTime::from_timestamp(row.get(4).unwrap(), 0),
            created_at: NaiveDateTime::from_timestamp(row.get(5).unwrap(), 0),
            updated_at: NaiveDateTime::from_timestamp(row.get(6).unwrap(), 0),
        }
    }
}
