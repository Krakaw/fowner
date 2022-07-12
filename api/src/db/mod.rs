mod migrations;
pub mod models;
pub mod processor;

use crate::FownerError;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::rusqlite::{Statement, Transaction};
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
                    PRAGMA journal_mode = WAL;
                    PRAGMA synchronous = normal;
                    PRAGMA temp_store = memory;
                    PRAGMA mmap_size = 30000000000;
                    PRAGMA page_size = 32768;
                    PRAGMA foreign_keys = ON
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

pub enum Connection<'a> {
    Pooled(PooledConnection<SqliteConnectionManager>),
    Transaction(Transaction<'a>),
}

impl<'a> TryFrom<&Db> for Connection<'a> {
    type Error = FownerError;

    fn try_from(db: &Db) -> Result<Self, Self::Error> {
        Ok(Self::Pooled(db.pool.get()?))
    }
}

impl<'a> From<Transaction<'a>> for Connection<'a> {
    fn from(transaction: Transaction<'a>) -> Self {
        Self::Transaction(transaction)
    }
}
impl<'a> Connection<'a> {
    #[inline]
    pub fn prepare(&self, query: &str) -> Result<Statement, FownerError> {
        match self {
            Connection::Pooled(client) => Ok(client.prepare(query)?),
            Connection::Transaction(transaction) => Ok(transaction.prepare(query)?),
        }
    }

    #[inline]
    pub fn last_insert_rowid(&self) -> i64 {
        match self {
            Connection::Pooled(client) => client.last_insert_rowid(),
            Connection::Transaction(transaction) => transaction.last_insert_rowid(),
        }
    }

    pub fn transaction(self) -> Result<Transaction<'a>, FownerError> {
        match self {
            Connection::Pooled(_) => Err(FownerError::Internal("Not a transaction".to_string())),
            Connection::Transaction(t) => Ok(t),
        }
    }
}
