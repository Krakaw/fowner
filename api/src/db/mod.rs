mod migrations;
pub mod models;
pub mod processor;

use crate::FownerError;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;
use std::sync::Arc;

#[derive(Clone)]
pub struct Db {
    pub pool: Arc<Pool<SqliteConnectionManager>>,
}

impl Db {
    pub fn new(connection_string: &Path) -> Result<Self, FownerError> {
        let sqlite_connection_manager = SqliteConnectionManager::file(connection_string);
        let sqlite_pool = r2d2::Pool::new(sqlite_connection_manager)?;
        let pool = Arc::new(sqlite_pool);
        Ok(Db { pool })
    }

    pub fn init(&self) -> Result<(), FownerError> {
        let connection = self.pool.get()?;
        let migrations = migrations::migrations();
        for migration in migrations {
            connection.execute_batch(migration)?;
        }
        Ok(())
    }
}