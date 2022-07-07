mod migrations;
pub mod models;
pub mod processor;

use crate::FownerError;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Db {
    pub pool: Arc<Pool<SqliteConnectionManager>>,
}

impl Db {
    pub fn new(connection_string: &Path) -> Result<Self, FownerError> {
        let sqlite_connection_manager =
            SqliteConnectionManager::file(connection_string).with_init(|c| {
                c.execute_batch(
                    r#"
                    pragma journal_mode = WAL;
                    pragma synchronous = normal;
                    pragma temp_store = memory;
                    pragma mmap_size = 30000000000;
                    pragma page_size = 32768;
                    "#,
                )
            });
        let sqlite_pool = Pool::new(sqlite_connection_manager)?;
        let pool = Arc::new(sqlite_pool);

        // Performance tuning

        Ok(Db { pool })
    }

    pub fn init(&self) -> Result<(), FownerError> {
        let mut connection = self.pool.get()?;
        let migrations = migrations::migrations();
        migrations.to_latest(&mut connection)?;
        Ok(())
    }
}
