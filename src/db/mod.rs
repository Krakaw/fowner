mod migrations;
pub mod models;

use crate::db::models::commit::Commit;
use crate::git::history::GitHistory;
use crate::GitRepo;
use anyhow::Result;
use r2d2::Pool;
use r2d2_sqlite::rusqlite::params;
use r2d2_sqlite::rusqlite::types::{FromSql, FromSqlResult, ValueRef};
use r2d2_sqlite::SqliteConnectionManager;
use std::path::PathBuf;
use std::sync::Arc;

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

    pub fn store_history(&self, repo: &GitRepo, since: Option<i64>) -> Result<Vec<GitHistory>> {
        let history = self.gather_data(repo, since)?;
        for row in history.clone() {
            row.store(self)?;
        }
        Ok(history)
    }

    pub fn gather_data(&self, repo: &GitRepo, since: Option<i64>) -> Result<Vec<GitHistory>> {
        let latest_commit = if let Some(since) = since {
            since
        } else {
            Commit::fetch_latest(self)
                .map(|c| c.commit_time.timestamp())
                .unwrap_or_else(|_| 0_i64)
        };
        let history = repo.parse((latest_commit + 1) as usize)?;
        Ok(history)
    }
}
