use crate::Db;
use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Owner {
    pub id: u32,
    pub handle: String,
    pub name: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NewOwner {
    pub handle: String,
    pub name: Option<String>,
}

impl Owner {
    pub fn load_by_handle(handle: String, db: &Db) -> Result<Self> {
        let absolute = fs::canonicalize(handle.clone())?;
        let absolute = absolute.to_string_lossy();
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, handle, name, created_at, updated_at FROM owners WHERE LOWER(handle) = LOWER(?);")?;
        let mut rows = stmt.query(params![absolute])?;
        if let Some(row) = rows.next()? {
            Ok(Owner::from(row))
        } else {
            Err(anyhow!("Owner not found"))
        }
    }
}

impl NewOwner {
    pub fn new(&self, db: &Db) -> Result<Owner> {
        if let Ok(owner) = Owner::load_by_handle(self.handle.clone(), db) {
            return Ok(owner);
        };
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("INSERT INTO owners (handle, name, created_at, updated_at) VALUES (?, ?, strftime('%s','now'), strftime('%s','now'))")?;
        let _res = stmt.execute(params![self.handle, self.name])?;
        Owner::load_by_handle(self.handle.clone(), db)
    }
}

impl<'stmt> From<&Row<'stmt>> for Owner {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            handle: row.get(1).unwrap(),
            name: row.get(2).unwrap(),
            created_at: NaiveDateTime::from_timestamp(row.get(4).unwrap(), 0),
            updated_at: NaiveDateTime::from_timestamp(row.get(5).unwrap(), 0),
        }
    }
}
