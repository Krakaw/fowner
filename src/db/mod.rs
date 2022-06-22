mod migrations;
pub mod models;
pub mod processor;

use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::rusqlite::params;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct Db {
    pub pool: Arc<Pool<SqliteConnectionManager>>,
}

impl Db {
    pub fn new(connection_string: &PathBuf) -> Result<Self> {
        let sqlite_connection_manager = SqliteConnectionManager::file(connection_string);
        let sqlite_pool = r2d2::Pool::new(sqlite_connection_manager)?;
        let pool = Arc::new(sqlite_pool);
        Ok(Db { pool })
    }

    pub fn init(&self) -> Result<()> {
        let connection = self.pool.get()?;
        let migrations = migrations::migrations();
        for migration in migrations {
            connection.execute(migration, params![])?;
        }
        Ok(())
    }
}
