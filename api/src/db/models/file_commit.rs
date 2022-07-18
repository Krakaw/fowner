use r2d2_sqlite::rusqlite::params;

use crate::db::models::commit::Commit;
use crate::db::Connection;
use crate::errors::FownerError;
use crate::File;

#[derive(Clone)]
pub struct FileCommit {
    pub file_id: u32,
    pub commit_id: u32,
}

impl FileCommit {
    pub fn save(&self, conn: &Connection) -> Result<Self, FownerError> {
        let sql = "INSERT INTO file_commits (file_id, commit_id) VALUES(?1, ?2)";
        let mut stmt = conn.prepare(sql)?;
        let _res = stmt.execute(params![self.file_id, self.commit_id,])?;
        Ok(self.clone())
    }

    pub fn fetch_between(
        project_id: u32,
        from_commit: Commit,
        to_commit: Commit,
        conn: &Connection,
    ) -> Result<Vec<File>, FownerError> {
        let sql = r#"
        SELECT f.id
        FROM commits c
                 INNER JOIN file_commits fc ON fc.commit_id = c.id
                 INNER JOIN files f ON f.id = fc.file_id
        WHERE c.commit_time BETWEEN ?1 AND ?2
          AND c.project_id = ?3
        GROUP BY f.id;
          "#;
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(
            params![
                from_commit.commit_time.timestamp(),
                to_commit.commit_time.timestamp(),
                from_commit.project_id
            ],
            |r| r.get(0),
        )?;
        let mut file_ids: Vec<u32> = vec![];
        for row in rows {
            file_ids.push(row?)
        }
        let ids = file_ids
            .iter()
            .map(|r| format!("{}", r.to_string()))
            .collect::<Vec<String>>()
            .join(",");
        let mut stmt = conn.prepare(&File::sql(Some(format!("AND f.id IN ({})", ids)), None))?;
        let rows = stmt.query_map(params![project_id,], |r| Ok(File::from(r)))?;
        let mut files = vec![];
        for row in rows {
            files.push(row?)
        }
        Ok(files)
    }
}
