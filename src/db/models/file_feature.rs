use crate::errors::FownerError;
use crate::Db;
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};

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
    pub fn load(file_id: u32, feature_id: u32, db: &Db) -> Result<FileFeature, FownerError> {
        let sql = "SELECT file_id, feature_id, created_at, updated_at FROM file_features WHERE file_id = ?1 AND feature_id = ?2";
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query(params![file_id, feature_id])?;
        if let Some(row) = rows.next()? {
            Ok(FileFeature::from(row))
        } else {
            Err(FownerError::NotFound)
        }
    }
}

impl NewFileFeature {
    pub fn save(&self, db: &Db) -> Result<FileFeature, FownerError> {
        let sql = "INSERT OR IGNORE INTO file_features (file_id, feature_id, created_at, updated_at) VALUES (?1, ?2, strftime('%s','now'), strftime('%s','now'))";
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare(sql)?;
        let _res = stmt.execute(params![self.file_id, self.feature_id,])?;
        FileFeature::load(self.file_id, self.feature_id, db)
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
