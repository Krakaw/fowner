use crate::errors::FownerError;
use crate::errors::FownerError::NotFound;
use crate::Db;
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

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
    pub fn search_by_handle(handle: String, db: &Db) -> Result<Vec<Owner>, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, handle, name, created_at, updated_at FROM owners WHERE handle LIKE ?1;",
        )?;
        let rows = stmt.query_map(params![format!("%{}%", handle)], |r| Ok(Owner::from(r)))?;
        let mut result = vec![];
        for row in rows {
            result.push(row?);
        }
        Ok(result)
    }
    pub fn load_by_handle(handle: String, db: &Db) -> Result<Self, FownerError> {
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("SELECT id, handle, name, created_at, updated_at FROM owners WHERE LOWER(handle) = LOWER(?1);")?;
        let mut rows = stmt.query(params![handle])?;
        if let Some(row) = rows.next()? {
            Ok(Owner::from(row))
        } else {
            Err(NotFound)
        }
    }
}

impl NewOwner {
    pub fn save(&self, db: &Db) -> Result<Owner, FownerError> {
        if let Ok(owner) = Owner::load_by_handle(self.handle.clone(), db) {
            return Ok(owner);
        };
        let conn = db.pool.get()?;
        let mut stmt = conn.prepare("INSERT INTO owners (handle, name, created_at, updated_at) VALUES (?1, ?2, strftime('%s','now'), strftime('%s','now'))")?;
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
            created_at: NaiveDateTime::from_timestamp(row.get(3).unwrap(), 0),
            updated_at: NaiveDateTime::from_timestamp(row.get(4).unwrap(), 0),
        }
    }
}
