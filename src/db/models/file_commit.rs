use crate::db::models::commit::Commit;
use crate::errors::FownerError;
use crate::{Db, File};
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

    pub fn fetch_between(
        from_commit: Commit,
        to_commit: Commit,
        db: &Db,
    ) -> Result<Vec<File>, FownerError> {
        let sql = r#"
        SELECT *
        FROM commits c
                 LEFT JOIN file_commits fc ON fc.commit_id = c.id
                 LEFT JOIN file_features ff ON ff.file_id = fc.file_id
                 LEFT JOIN files f ON f.id = ff.file_id
        WHERE c.commit_time BETWEEN ?1 AND ?2
          AND c.project_id = ?3;
          "#;
        Ok(vec![])
    }
}
