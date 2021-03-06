use crate::db::models::{extract_all, extract_first};
use crate::db::Connection;
use crate::errors::FownerError;
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Feature {
    pub id: u32,
    pub project_id: u32,
    pub name: String,
    pub description: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

pub struct NewFeature {
    pub project_id: u32,
    pub name: String,
    pub description: Option<String>,
}

impl NewFeature {
    pub fn save(&self, conn: &Connection) -> Result<Feature, FownerError> {
        if let Ok(feature) = Feature::load_by_name(self.project_id, self.name.clone(), conn) {
            return Ok(feature);
        }
        let mut stmt = conn.prepare("INSERT INTO features (project_id, name, description, created_at, updated_at) VALUES (?1, ?2, ?3, strftime('%s','now'), strftime('%s','now'))")?;
        let _res = stmt.execute(params![
            self.project_id.clone(),
            self.name.clone(),
            self.description.clone()
        ])?;
        Feature::load(conn.last_insert_rowid() as u32, conn)
    }
}
impl Feature {
    fn sql(where_clause: Option<String>) -> String {
        format!(
            r#"
        SELECT id, project_id, name, description, created_at, updated_at
            FROM features
            {}
        "#,
            where_clause.unwrap_or_default()
        )
    }
    pub fn load(id: u32, conn: &Connection) -> Result<Feature, FownerError> {
        let mut stmt = conn.prepare(&Feature::sql(Some("WHERE id = ?1".to_string())))?;
        extract_first!(params![id], stmt)
    }
    pub fn load_by_name(
        project_id: u32,
        name: String,
        conn: &Connection,
    ) -> Result<Feature, FownerError> {
        let mut stmt = conn.prepare(&Feature::sql(Some(
            "WHERE project_id = ?1 AND name LIKE ?2".to_string(),
        )))?;
        extract_first!(params![project_id, name], stmt)
    }
    pub fn load_by_project(
        project_id: u32,
        conn: &Connection,
    ) -> Result<Vec<Feature>, FownerError> {
        let mut stmt = conn.prepare(&Feature::sql(Some("WHERE project_id = ?1;".to_string())))?;
        extract_all!(params![project_id], stmt)
    }
}

impl<'stmt> From<&Row<'stmt>> for Feature {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            project_id: row.get(1).unwrap(),
            name: row.get(2).unwrap(),
            description: row.get(3).unwrap(),
            created_at: NaiveDateTime::from_timestamp(row.get(4).unwrap(), 0),
            updated_at: NaiveDateTime::from_timestamp(row.get(5).unwrap(), 0),
        }
    }
}
