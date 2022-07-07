use crate::db::models::commit::Commit;
use crate::db::models::extract_first;
use crate::db::models::feature::Feature;
use crate::db::Connection;
use crate::errors::FownerError;
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};

#[derive(Debug)]
pub struct FileFeature {
    pub file_id: u32,
    pub feature_id: u32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub struct NewFileFeature {
    pub file_id: u32,
    pub feature_id: u32,
}

impl FileFeature {
    pub fn load(
        file_id: u32,
        feature_id: u32,
        conn: &Connection,
    ) -> Result<FileFeature, FownerError> {
        let sql = "SELECT file_id, feature_id, created_at, updated_at FROM file_features WHERE file_id = ?1 AND feature_id = ?2";
        let mut stmt = conn.prepare(sql)?;
        extract_first!(params![file_id, feature_id], stmt)
    }

    pub fn fetch_between(
        from_commit: Commit,
        to_commit: Commit,
        conn: &Connection,
    ) -> Result<Vec<Feature>, FownerError> {
        let sql = r#"
        SELECT f.*
        FROM commits c
                 INNER JOIN file_commits fc ON fc.commit_id = c.id
                 INNER JOIN file_features ff ON ff.file_id = fc.file_id
                 INNER JOIN features f ON f.id = ff.feature_id
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
            |r| Ok(Feature::from(r)),
        )?;
        let mut result = vec![];
        for row in rows {
            result.push(row?)
        }
        Ok(result)
    }
}

impl NewFileFeature {
    pub fn save(&self, conn: &Connection) -> Result<FileFeature, FownerError> {
        let sql = "INSERT OR IGNORE INTO file_features (file_id, feature_id, created_at, updated_at) VALUES (?1, ?2, strftime('%s','now'), strftime('%s','now'))";
        let mut stmt = conn.prepare(sql)?;
        let _res = stmt.execute(params![self.file_id, self.feature_id,])?;
        FileFeature::load(self.file_id, self.feature_id, conn)
    }
}

impl<'stmt> From<&Row<'stmt>> for FileFeature {
    fn from(row: &Row) -> Self {
        Self {
            file_id: row.get(0).unwrap(),
            feature_id: row.get(1).unwrap(),
            created_at: NaiveDateTime::from_timestamp(row.get(2).unwrap(), 0),
            updated_at: NaiveDateTime::from_timestamp(row.get(3).unwrap(), 0),
        }
    }
}
