use crate::errors::FownerError;
use crate::Db;
use r2d2_sqlite::rusqlite::params;

#[derive(Clone)]
pub struct FileCommit {
    pub file_id: u32,
    pub commit_id: u32,
}

impl FileCommit {
    pub fn save(&self, db: &Db) -> Result<Self, FownerError> {
        let sql = "INSERT INTO file_commits (file_id, commit_id) VALUES(?1, ?2)";
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare(sql)?;
        let _res = stmt.execute(params![self.file_id, self.commit_id,])?;
        Ok(self.clone())
    }
}
