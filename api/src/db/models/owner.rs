use crate::db::models::{extract_all, extract_first};
use crate::db::Connection;
use crate::errors::FownerError;
use crate::Db;
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Owner {
    pub id: u32,
    pub handle: String,
    pub name: Option<String>,
    pub primary_owner_id: Option<u32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NewOwner {
    pub handle: String,
    pub name: Option<String>,
    pub primary_owner_id: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UpdateOwner {
    pub name: Option<String>,
    pub primary_owner_id: Option<u32>,
}

impl Owner {
    pub fn search_by_handle(handle: String, db: &Db) -> Result<Vec<Self>, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, handle, name, primary_owner_id, created_at, updated_at FROM owners WHERE handle LIKE ?1;",
        )?;
        extract_all!(params![format!("%{}%", handle)], stmt)
    }

    pub fn load(id: u32, db: &Db) -> Result<Self, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, handle, name, primary_owner_id, created_at, updated_at FROM owners WHERE id = ?1;")?;
        extract_first!(params![id], stmt)
    }

    pub fn load_by_handle(handle: String, conn: &Connection) -> Result<Self, FownerError> {
        let mut stmt = conn.prepare("SELECT id, handle, name, primary_owner_id, created_at, updated_at FROM owners WHERE LOWER(handle) = LOWER(?1);")?;
        extract_first!(params![handle], stmt)
    }

    pub fn update(self, update_details: UpdateOwner, db: &Db) -> Result<Self, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt =
            conn.prepare("UPDATE owners SET name = ?1, primary_owner_id = ?2 WHERE id = ?3")?;
        let _res = stmt.execute(params![
            update_details.name,
            update_details.primary_owner_id,
            self.id
        ])?;
        Self::load(self.id, db)
    }
}

impl NewOwner {
    pub fn save(&self, conn: &Connection) -> Result<Owner, FownerError> {
        if let Ok(owner) = Owner::load_by_handle(self.handle.clone(), conn) {
            return Ok(owner);
        };
        let mut stmt = conn.prepare("INSERT INTO owners (handle, name, primary_owner_id, created_at, updated_at) VALUES (?1, ?2, ?3, strftime('%s','now'), strftime('%s','now'))")?;
        let _res = stmt.execute(params![self.handle, self.name, self.primary_owner_id])?;
        Owner::load_by_handle(self.handle.clone(), conn)
    }
}

impl<'stmt> From<&Row<'stmt>> for Owner {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            handle: row.get(1).unwrap(),
            name: row.get(2).unwrap(),
            primary_owner_id: row.get(3).unwrap(),
            created_at: NaiveDateTime::from_timestamp(row.get(4).unwrap(), 0),
            updated_at: NaiveDateTime::from_timestamp(row.get(5).unwrap(), 0),
        }
    }
}
