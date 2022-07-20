use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};
use serde::{Deserialize, Serialize};

use crate::db::models::{extract_all, extract_first};
use crate::db::Connection;
use crate::errors::FownerError;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
    pub fn sql(where_sql: &str) -> String {
        format!(
            "SELECT id, handle, name, primary_owner_id, created_at, updated_at FROM owners WHERE {}",
            where_sql
        )
    }

    pub fn all(conn: &Connection) -> Result<Vec<Self>, FownerError> {
        let mut stmt = conn.prepare(&Self::sql("1 = 1"))?;
        extract_all!(params![], stmt)
    }

    pub fn search_by_handle(handle: String, conn: &Connection) -> Result<Vec<Self>, FownerError> {
        let mut stmt = conn.prepare(&Self::sql("handle LIKE ?1"))?;
        extract_all!(params![format!("%{}%", handle)], stmt)
    }

    pub fn load(id: u32, conn: &Connection) -> Result<Self, FownerError> {
        let mut stmt = conn.prepare(&Self::sql("id = ?1"))?;
        extract_first!(params![id], stmt)
    }

    pub fn load_by_handle(handle: String, conn: &Connection) -> Result<Self, FownerError> {
        let mut stmt = conn.prepare(&Self::sql("LOWER(handle) = LOWER(?1)"))?;
        extract_first!(params![handle], stmt)
    }

    pub fn update(
        self,
        update_details: UpdateOwner,
        conn: &Connection,
    ) -> Result<Self, FownerError> {
        let mut stmt =
            conn.prepare("UPDATE owners SET name = ?1, primary_owner_id = ?2, updated_at = strftime('%s','now') WHERE id = ?3")?;
        let _res = stmt.execute(params![
            update_details.name,
            update_details.primary_owner_id,
            self.id
        ])?;
        Self::load(self.id, conn)
    }
}

impl NewOwner {
    pub fn save_or_load(&self, conn: &Connection) -> Result<Owner, FownerError> {
        if let Ok(owner) = Owner::load_by_handle(self.handle.clone(), conn) {
            return Ok(owner);
        };
        self.save(conn)
    }

    pub fn save(&self, conn: &Connection) -> Result<Owner, FownerError> {
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

#[cfg(test)]
mod tests {
    use crate::db::models::owner::{NewOwner, Owner, UpdateOwner};
    use crate::test::tests::TestHandler;
    use crate::Connection;

    #[test]
    fn load() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = Connection::try_from(db).unwrap();
        let owner = NewOwner {
            handle: "Krakaw".to_string(),
            name: Some("Kra-kaw!".to_string()),
            primary_owner_id: None,
        }
        .save(&conn)
        .unwrap();

        let db_owner = Owner::load(owner.id, &conn).unwrap();
        assert_eq!(owner, db_owner);
    }

    #[test]
    fn load_by_handle() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = Connection::try_from(db).unwrap();
        let owner = NewOwner {
            handle: "Krakaw".to_string(),
            name: Some("Kra-kaw!".to_string()),
            primary_owner_id: None,
        }
        .save(&conn)
        .unwrap();

        let db_owner = Owner::load_by_handle("KraKaw".to_string(), &conn).unwrap();
        assert_eq!(owner, db_owner);

        let db_owner = Owner::load_by_handle("unknown".to_string(), &conn);
        assert!(db_owner.is_err());
    }

    #[test]
    fn search_by_handle() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = Connection::try_from(db).unwrap();
        let owner = NewOwner {
            handle: "Krakaw".to_string(),
            name: Some("Kra-kaw!".to_string()),
            primary_owner_id: None,
        }
        .save(&conn)
        .unwrap();

        let db_owner = Owner::search_by_handle("kra".to_string(), &conn).unwrap();
        assert_eq!(owner, db_owner[0]);
    }

    #[test]
    fn all() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = Connection::try_from(db).unwrap();
        let owner_1 = NewOwner {
            handle: "Krakaw".to_string(),
            name: Some("Kra-kaw!".to_string()),
            primary_owner_id: None,
        }
        .save(&conn)
        .unwrap();
        let owner_2 = NewOwner {
            handle: "Krakaw2".to_string(),
            name: Some("Kra-kaw! 2".to_string()),
            primary_owner_id: Some(1),
        }
        .save(&conn)
        .unwrap();

        let all = Owner::all(&conn).unwrap();
        assert_eq!(all, vec![owner_1, owner_2]);
    }

    #[test]
    fn update() {
        let handler = TestHandler::init();
        let db = &handler.db;
        let conn = Connection::try_from(db).unwrap();
        let owner = NewOwner {
            handle: "Krakaw".to_string(),
            name: Some("Kra-kaw!".to_string()),
            primary_owner_id: None,
        }
        .save(&conn)
        .unwrap();

        let db_owner = Owner::load(owner.id, &conn).unwrap();
        assert_eq!(owner, db_owner);

        let new_owner = owner
            .update(
                UpdateOwner {
                    name: Some("krakaw".to_string()),
                    primary_owner_id: None,
                },
                &conn,
            )
            .unwrap();
        assert_eq!(new_owner.id, db_owner.id);
        assert_eq!(new_owner.name, Some("krakaw".to_string()));
    }
}
