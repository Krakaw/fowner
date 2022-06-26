use crate::db::models::feature::NewFeature;
use crate::db::models::file_feature::{FileFeature, NewFileFeature};
use crate::db::models::file_owner::FileOwner;
use crate::errors::FownerError;
use crate::Db;
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    pub id: u32,
    pub project_id: u32,
    pub path: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub feature_names: Vec<String>,
}

impl File {
    fn sql(where_clause: Option<String>) -> String {
        format!(
            r#"
        SELECT f.id, f.project_id, f.path, f.created_at, f.updated_at, GROUP_CONCAT(fe.name, ',') AS feature_names
        FROM files f
                 LEFT JOIN file_features ff on f.id = ff.file_id
                 LEFT JOIN features fe on ff.feature_id = fe.id
        WHERE f.project_id = ?1
        {}
        GROUP BY f.id;
        "#,
            where_clause.unwrap_or_default()
        )
    }
    pub fn all(project_id: u32, db: &Db) -> Result<Vec<File>, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare(&File::sql(None))?;
        let rows = stmt.query_map(params![project_id], |r| Ok(File::from(r)))?;
        let mut result = vec![];
        for row in rows {
            result.push(row?)
        }

        Ok(result)
    }

    pub fn load_by_path(project_id: u32, path: String, db: &Db) -> Result<File, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare(&File::sql(Some("AND path = ?2".to_string())))?;
        let mut rows = stmt.query(params![project_id, path])?;
        if let Some(row) = rows.next()? {
            Ok(File::from(row))
        } else {
            Err(FownerError::NotFound("File not found".to_string()))
        }
    }

    pub fn get_owners(&self, db: &Db) -> Result<Vec<FileOwner>, FownerError> {
        FileOwner::load(self.id, None, None, db)
    }

    pub fn add_feature(&self, feature_id: u32, db: &Db) -> Result<FileFeature, FownerError> {
        NewFileFeature {
            file_id: self.id,
            feature_id,
        }
        .save(db)
    }

    pub fn generate_feature_file(
        project_id: u32,
        dotfile: PathBuf,
        db: &Db,
    ) -> Result<PathBuf, FownerError> {
        // Load any existing file
        let existing_contents = if dotfile.exists() {
            std::fs::read_to_string(dotfile.clone())?
        } else {
            String::new()
        };
        let existing_path_features = existing_contents
            .split('\n')
            .filter_map(|r| {
                let row = r.trim();
                if !row.is_empty() {
                    let parts = row.split('|').collect::<Vec<&str>>();
                    return Some((
                        parts.get(0).cloned().unwrap(),
                        parts
                            .get(1)
                            .cloned()
                            .unwrap()
                            .split(',')
                            .collect::<Vec<&str>>(),
                    ));
                }
                None
            })
            .collect::<Vec<(&str, Vec<&str>)>>();

        let files = Self::all(project_id, db)?;
        for existing_row in existing_path_features {
            let db_file =
                if let Some(db_file) = files.iter().find(|r| r.path == existing_row.0).cloned() {
                    db_file
                } else {
                    // Create the file
                    NewFile {
                        project_id,
                        path: existing_row.0.to_string(),
                    }
                    .save(db)?
                };
            for feature_str in existing_row.1 {
                // Check if the features exist, if not create them and attach them to the File
                let feature = NewFeature {
                    project_id,
                    name: feature_str.to_string(),
                    description: None,
                }
                .save(db)?;
                db_file.add_feature(feature.id, db)?;
            }
        }
        let mut files = Self::all(project_id, db)?;
        files.sort_by(|a, b| a.path.cmp(&b.path));
        std::fs::write(
            dotfile.clone(),
            files
                .iter()
                .map(|r| format!("{}|{}", r.path.clone(), r.feature_names.join(",")))
                .collect::<Vec<String>>()
                .join("\n"),
        )?;
        Ok(dotfile)
    }
}
pub struct NewFile {
    pub project_id: u32,
    pub path: String,
}

impl NewFile {
    pub fn save(&self, db: &Db) -> Result<File, FownerError> {
        if let Ok(file) = File::load_by_path(self.project_id, self.path.clone(), db) {
            return Ok(file);
        };
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("INSERT INTO files (project_id, path, created_at, updated_at) VALUES (?1, ?2, strftime('%s','now'), strftime('%s','now'))")?;
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
            feature_names: row
                .get(5)
                .map(|s: String| s.split(',').map(|s| s.to_string()).collect())
                .unwrap_or_default(),
        }
    }
}
